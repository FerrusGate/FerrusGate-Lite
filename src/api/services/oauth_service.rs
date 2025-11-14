use actix_web::{HttpResponse, web};
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::cache::CompositeCache;
use crate::errors::AppError;
use crate::security::{JwtManager, generate_random_token};
use crate::storage::{ClientRepository, SeaOrmBackend, TokenRepository};

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
/// 生成授权码（简化版，实际应该先显示授权页面让用户确认）
pub async fn authorize(
    query: web::Query<AuthorizeRequest>,
    storage: web::Data<Arc<SeaOrmBackend>>,
    cache: web::Data<Arc<CompositeCache>>,
    config: web::Data<crate::config::AppConfig>,
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

    // 4. 这里应该验证用户身份并让用户确认授权
    // 简化版：假设用户已登录且同意授权，使用 user_id = 1（需要实际实现 session 管理）
    // TODO: 实现真实的用户会话验证和授权确认页面
    let user_id = 1; // 临时硬编码

    // 5. 生成授权码
    let code = generate_random_token(32);

    // 6. 计算过期时间
    let expires_at = Utc::now() + Duration::seconds(config.auth.authorization_code_expire);

    // 7. 保存授权码到数据库
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

    // 8. 缓存授权码（用于快速验证）
    cache
        .set(
            &format!("authcode:{}", code),
            "valid".to_string(),
            Some(config.auth.authorization_code_expire as u64),
        )
        .await;

    tracing::info!(
        "Authorization code generated for client: {} user: {}",
        client.name,
        user_id
    );

    // 9. 构造重定向 URL
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
    config: web::Data<crate::config::AppConfig>,
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

    // 6. 生成 access_token 和 refresh_token
    let access_token = jwt_manager.generate_token(
        auth_data.user_id as i64,
        config.auth.access_token_expire,
        Some(parse_scopes(&auth_data.scopes)),
    )?;

    let refresh_token = jwt_manager.generate_token(
        auth_data.user_id as i64,
        config.auth.refresh_token_expire,
        Some(vec!["refresh".to_string()]),
    )?;

    // 7. 保存 token 到数据库
    let access_token_id = storage
        .save_access_token(
            &access_token,
            &auth_data.client_id,
            auth_data.user_id,
            &auth_data.scopes,
            Utc::now() + Duration::seconds(config.auth.access_token_expire),
        )
        .await?;

    storage
        .save_refresh_token(
            &refresh_token,
            access_token_id,
            Utc::now() + Duration::seconds(config.auth.refresh_token_expire),
        )
        .await?;

    // 8. 缓存 token
    cache
        .set(
            &format!("token:{}", access_token),
            auth_data.user_id.to_string(),
            Some(config.auth.access_token_expire as u64),
        )
        .await;

    // 9. 删除授权码缓存
    cache.delete(&format!("authcode:{}", code)).await;

    tracing::info!(
        "Access token issued for client: {} user: {}",
        client.name,
        auth_data.user_id
    );

    Ok(HttpResponse::Ok().json(TokenResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: config.auth.access_token_expire,
        id_token: None, // TODO: 实现 OIDC ID Token
    }))
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
