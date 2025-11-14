use actix_web::{HttpMessage, HttpRequest, HttpResponse, web};
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::cache::CompositeCache;
use crate::errors::AppError;
use crate::security::{Claims, JwtManager, generate_random_token};
use crate::storage::{ClientRepository, SeaOrmBackend, TokenRepository, UserRepository};

#[derive(Debug, Deserialize)]
pub struct AuthorizeRequest {
    pub response_type: String,
    pub client_id: String,
    pub redirect_uri: String,
    pub scope: Option<String>,
    pub state: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AuthorizeResponse {
    pub code: String,
    pub state: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TokenRequest {
    pub grant_type: String,
    pub code: Option<String>,
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
}

#[derive(Debug, Serialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id_token: Option<String>,
}

/// GET /oauth/authorize
/// 生成授权码（需要用户已登录）
pub async fn authorize(
    req: HttpRequest,
    query: web::Query<AuthorizeRequest>,
    storage: web::Data<Arc<SeaOrmBackend>>,
    cache: web::Data<Arc<CompositeCache>>,
    jwt_manager: web::Data<Arc<JwtManager>>,
) -> Result<HttpResponse, AppError> {
    // 1. 验证 response_type
    if query.response_type != "code" {
        return Err(AppError::BadRequest("Unsupported response_type".into()));
    }

    // 2. 验证 client_id
    let client = storage
        .find_by_client_id(&query.client_id)
        .await?
        .ok_or(AppError::InvalidClient)?;

    // 3. 验证 redirect_uri
    if !storage
        .verify_redirect_uri(&query.client_id, &query.redirect_uri)
        .await?
    {
        return Err(AppError::InvalidRedirectUri);
    }

    // 4. 从请求中提取用户身份（通过 JWT token 或 session）
    // 优先从 Authorization header 获取 JWT
    let user_id = if let Some(auth_header) = req.headers().get("Authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                // 验证并解析 JWT
                let claims = jwt_manager.verify_token(token)?;

                // 检查黑名单
                if cache.exists(&format!("blacklist:{}", token)).await {
                    return Err(AppError::Unauthorized);
                }

                claims
                    .sub
                    .parse::<i64>()
                    .map_err(|_| AppError::Internal("Invalid user_id in token".into()))?
            } else {
                return Err(AppError::Unauthorized);
            }
        } else {
            return Err(AppError::Unauthorized);
        }
    } else {
        // 如果没有 Authorization header，尝试从请求扩展中获取（如果使用了认证中间件）
        let claims = req
            .extensions()
            .get::<Claims>()
            .cloned()
            .ok_or_else(|| AppError::Unauthorized)?;

        claims
            .sub
            .parse::<i64>()
            .map_err(|_| AppError::Internal("Invalid user_id in token".into()))?
    };

    // 验证用户是否存在
    storage
        .find_by_id(user_id)
        .await?
        .ok_or(AppError::NotFound)?;

    // 5. 读取认证策略配置（从数据库）
    let auth_policy = storage.get_auth_policy_config().await?;

    // 6. 生成授权码
    let code = generate_random_token(32);

    // 7. 计算过期时间
    let expires_at = Utc::now() + Duration::seconds(auth_policy.authorization_code_expire);

    // 8. 保存授权码到数据库
    storage
        .save_auth_code(
            &code,
            &query.client_id,
            user_id,
            &query.redirect_uri,
            &query.scope.clone().unwrap_or_default(),
            expires_at,
        )
        .await?;

    // 9. 缓存授权码（用于快速验证）
    cache
        .set(
            &format!("authcode:{}", code),
            "valid".to_string(),
            Some(auth_policy.authorization_code_expire as u64),
        )
        .await;

    tracing::info!(
        "Authorization code generated for client: {} user: {}",
        client.name,
        user_id
    );

    // 10. 构造重定向 URL
    let mut redirect_url = format!("{}?code={}", query.redirect_uri, code);
    if let Some(ref state) = query.state {
        redirect_url.push_str(&format!("&state={}", state));
    }

    // 返回 307 重定向
    Ok(HttpResponse::TemporaryRedirect()
        .insert_header(("Location", redirect_url))
        .finish())
}

/// POST /oauth/token
/// 授权码换取 access token
pub async fn token(
    req: web::Json<TokenRequest>,
    storage: web::Data<Arc<SeaOrmBackend>>,
    jwt_manager: web::Data<Arc<JwtManager>>,
    cache: web::Data<Arc<CompositeCache>>,
) -> Result<HttpResponse, AppError> {
    // 1. 验证 grant_type
    if req.grant_type != "authorization_code" {
        return Err(AppError::InvalidGrantType);
    }

    // 2. 验证授权码
    let code = req
        .code
        .as_ref()
        .ok_or(AppError::BadRequest("Missing code".into()))?;

    let auth_data = storage
        .consume_auth_code(code)
        .await?
        .ok_or(AppError::InvalidAuthCode)?;

    // 3. 验证授权码是否过期
    if auth_data.expires_at < Utc::now() {
        return Err(AppError::InvalidAuthCode);
    }

    // 4. 验证 client_id 和 client_secret
    let client = storage
        .find_by_client_id(&req.client_id)
        .await?
        .ok_or(AppError::InvalidClient)?;

    if client.client_secret != req.client_secret {
        return Err(AppError::InvalidClient);
    }

    // 5. 验证 redirect_uri 与授权码中的一致
    if auth_data.redirect_uri != req.redirect_uri {
        return Err(AppError::InvalidRedirectUri);
    }

    // 6. 查询用户获取 role
    let user = storage
        .find_by_id(auth_data.user_id)
        .await?
        .ok_or(AppError::NotFound)?;

    // 7. 读取认证策略配置（从数据库）
    let auth_policy = storage.get_auth_policy_config().await?;

    // 8. 生成 access_token 和 refresh_token
    let access_token = jwt_manager.generate_token(
        auth_data.user_id as i64,
        auth_policy.access_token_expire,
        Some(parse_scopes(&auth_data.scopes)),
        &user.role,
    )?;

    let refresh_token = jwt_manager.generate_token(
        auth_data.user_id as i64,
        auth_policy.refresh_token_expire,
        Some(vec!["refresh".to_string()]),
        &user.role,
    )?;

    // 9. 保存 token 到数据库
    let access_token_id = storage
        .save_access_token(
            &access_token,
            &auth_data.client_id,
            auth_data.user_id,
            &auth_data.scopes,
            Utc::now() + Duration::seconds(auth_policy.access_token_expire),
        )
        .await?;

    storage
        .save_refresh_token(
            &refresh_token,
            access_token_id,
            Utc::now() + Duration::seconds(auth_policy.refresh_token_expire),
        )
        .await?;

    // 10. 缓存 token
    cache
        .set(
            &format!("token:{}", access_token),
            auth_data.user_id.to_string(),
            Some(auth_policy.access_token_expire as u64),
        )
        .await;

    // 11. 删除授权码缓存
    cache.delete(&format!("authcode:{}", code)).await;

    // 12. 生成 OIDC ID Token（如果 scope 包含 openid）
    let id_token = if auth_data.scopes.contains("openid") {
        Some(generate_id_token(
            &user,
            &auth_data.client_id,
            &jwt_manager,
            auth_policy.access_token_expire,
        )?)
    } else {
        None
    };

    tracing::info!(
        "Access token issued for client: {} user: {}",
        client.name,
        auth_data.user_id
    );

    Ok(HttpResponse::Ok().json(TokenResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: auth_policy.access_token_expire,
        id_token,
    }))
}

/// 生成 OIDC ID Token
fn generate_id_token(
    user: &crate::storage::entities::users::Model,
    client_id: &str,
    jwt_manager: &JwtManager,
    expires_in: i64,
) -> Result<String, AppError> {
    use jsonwebtoken::{EncodingKey, Header, encode};
    use serde_json::json;

    let now = Utc::now();
    let exp = (now + Duration::seconds(expires_in)).timestamp();
    let iat = now.timestamp();

    // 构造 ID Token claims
    let claims = json!({
        "iss": "ferrusgate",  // Issuer
        "sub": user.id.to_string(),  // Subject (user_id)
        "aud": client_id,  // Audience (client_id)
        "exp": exp,  // Expiration time
        "iat": iat,  // Issued at
        "name": user.username,  // User name
        "email": user.email,  // User email
        "email_verified": true,  // Email verification status
    });

    // 使用 JWT manager 的密钥生成 token
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_manager.secret().as_bytes()),
    )
    .map_err(|e| AppError::Internal(format!("Failed to generate ID token: {}", e)))?;

    Ok(token)
}

/// 解析 scope 字符串为数组
fn parse_scopes(scopes: &str) -> Vec<String> {
    if scopes.is_empty() {
        return vec![];
    }

    // 尝试解析 JSON 数组
    if let Ok(parsed) = serde_json::from_str::<Vec<String>>(scopes) {
        return parsed;
    }

    // 否则按空格分割
    scopes.split_whitespace().map(|s| s.to_string()).collect()
}
