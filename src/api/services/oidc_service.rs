use actix_web::{HttpMessage, HttpRequest, HttpResponse, web};
use serde::Serialize;
use std::sync::Arc;

use crate::errors::AppError;
use crate::security::Claims;
use crate::storage::{SeaOrmBackend, UserRepository};

#[derive(Debug, Serialize)]
pub struct OpenIDConfiguration {
    pub issuer: String,
    pub authorization_endpoint: String,
    pub token_endpoint: String,
    pub userinfo_endpoint: String,
    pub jwks_uri: String,
    pub response_types_supported: Vec<String>,
    pub subject_types_supported: Vec<String>,
    pub id_token_signing_alg_values_supported: Vec<String>,
    pub scopes_supported: Vec<String>,
    pub token_endpoint_auth_methods_supported: Vec<String>,
    pub claims_supported: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct JWKSResponse {
    pub keys: Vec<JWK>,
}

#[derive(Debug, Serialize)]
pub struct JWK {
    pub kty: String,
    pub kid: String,
    pub r#use: String,
    pub alg: String,
    pub n: String,
    pub e: String,
}

#[derive(Debug, Serialize)]
pub struct UserInfoResponse {
    pub sub: String,
    pub name: String,
    pub email: String,
    pub email_verified: bool,
}

/// GET /.well-known/openid-configuration
/// OpenID Connect Discovery 文档
pub async fn discovery(config: web::Data<crate::config::AppConfig>) -> HttpResponse {
    let base_url = format!("http://{}:{}", config.server.host, config.server.port);

    let discovery = OpenIDConfiguration {
        issuer: base_url.clone(),
        authorization_endpoint: format!("{}/oauth/authorize", base_url),
        token_endpoint: format!("{}/oauth/token", base_url),
        userinfo_endpoint: format!("{}/oauth/userinfo", base_url),
        jwks_uri: format!("{}/.well-known/jwks.json", base_url),
        response_types_supported: vec!["code".to_string()],
        subject_types_supported: vec!["public".to_string()],
        id_token_signing_alg_values_supported: vec!["HS256".to_string()],
        scopes_supported: vec![
            "openid".to_string(),
            "profile".to_string(),
            "email".to_string(),
        ],
        token_endpoint_auth_methods_supported: vec!["client_secret_post".to_string()],
        claims_supported: vec![
            "sub".to_string(),
            "name".to_string(),
            "email".to_string(),
            "email_verified".to_string(),
        ],
    };

    HttpResponse::Ok().json(discovery)
}

/// GET /.well-known/jwks.json
/// JSON Web Key Set (公钥端点)
pub async fn jwks() -> HttpResponse {
    // TODO: 实现真实的 JWKS
    // 当前使用对称密钥 (HS256)，生产环境应该使用非对称密钥 (RS256)
    let jwks = JWKSResponse {
        keys: vec![
            // 对称密钥不应该公开，这里返回空数组
            // 如果要支持 ID Token 验证，需要切换到 RS256 并公开公钥
        ],
    };

    HttpResponse::Ok().json(jwks)
}

/// GET /oauth/userinfo
/// 获取用户信息（需要 Bearer Token）
pub async fn userinfo(
    req: HttpRequest,
    storage: web::Data<Arc<SeaOrmBackend>>,
) -> Result<HttpResponse, AppError> {
    // 从请求扩展中提取 Claims（由 JwtAuth 中间件注入）
    let claims = req
        .extensions()
        .get::<Claims>()
        .cloned()
        .ok_or(AppError::Unauthorized)?;

    // 解析 user_id
    let user_id: i32 = claims
        .sub
        .parse()
        .map_err(|_| AppError::Internal("Invalid user_id in token".into()))?;

    // 查询用户信息
    let user = storage
        .find_by_id(user_id)
        .await?
        .ok_or(AppError::NotFound)?;

    Ok(HttpResponse::Ok().json(UserInfoResponse {
        sub: user.id.to_string(),
        name: user.username.clone(),
        email: user.email.clone(),
        email_verified: true, // 简化版，实际应该有验证字段
    }))
}
