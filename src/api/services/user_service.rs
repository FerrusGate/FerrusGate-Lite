use actix_web::{HttpMessage, HttpRequest, HttpResponse, web};
use serde::Serialize;
use std::sync::Arc;

use crate::cache::CompositeCache;
use crate::errors::AppError;
use crate::security::Claims;
use crate::storage::{SeaOrmBackend, UserRepository};

#[derive(Debug, Serialize)]
pub struct UserProfileResponse {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct AuthorizationInfo {
    pub client_id: String,
    pub client_name: String,
    pub scopes: Vec<String>,
    pub granted_at: String,
}

/// GET /api/user/me
/// 获取当前用户信息
pub async fn get_profile(
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
    let user_id: i64 = claims
        .sub
        .parse()
        .map_err(|_| AppError::Internal("Invalid user_id in token".into()))?;

    // 查询用户信息
    let user = storage
        .find_by_id(user_id)
        .await?
        .ok_or(AppError::NotFound)?;

    Ok(HttpResponse::Ok().json(UserProfileResponse {
        id: user.id,
        username: user.username,
        email: user.email,
        created_at: user.created_at.to_rfc3339(),
    }))
}

/// GET /api/user/authorizations
/// 获取已授权的应用列表
pub async fn list_authorizations(
    req: HttpRequest,
    storage: web::Data<Arc<SeaOrmBackend>>,
) -> Result<HttpResponse, AppError> {
    // 从请求扩展中提取 Claims
    let claims = req
        .extensions()
        .get::<Claims>()
        .cloned()
        .ok_or(AppError::Unauthorized)?;

    let user_id: i64 = claims
        .sub
        .parse()
        .map_err(|_| AppError::Internal("Invalid user_id in token".into()))?;

    // 从数据库查询用户的授权记录
    let authorizations = storage.get_user_authorizations(user_id).await?;

    // 转换为 API 响应格式
    let response: Vec<AuthorizationInfo> = authorizations
        .into_iter()
        .map(|auth| AuthorizationInfo {
            client_id: auth.client_id,
            client_name: auth.client_name,
            scopes: auth.scopes,
            granted_at: auth.granted_at.to_rfc3339(),
        })
        .collect();

    Ok(HttpResponse::Ok().json(response))
}

/// DELETE /api/user/authorizations/{client_id}
/// 撤销对某个应用的授权
pub async fn revoke_authorization(
    req: HttpRequest,
    client_id: web::Path<String>,
    storage: web::Data<Arc<SeaOrmBackend>>,
    cache: web::Data<Arc<CompositeCache>>,
) -> Result<HttpResponse, AppError> {
    // 从请求扩展中提取 Claims
    let claims = req
        .extensions()
        .get::<Claims>()
        .cloned()
        .ok_or(AppError::Unauthorized)?;

    let user_id: i64 = claims
        .sub
        .parse()
        .map_err(|_| AppError::Internal("Invalid user_id in token".into()))?;

    // 从数据库删除授权记录（包括 access_tokens 和 refresh_tokens）
    storage
        .revoke_user_authorization(user_id, &client_id)
        .await?;

    // 将用户的相关 token 加入黑名单（可选，增加安全性）
    // 使用较短的 TTL，因为 token 本身有过期时间
    let blacklist_key = format!("blacklist:user:{}:client:{}", user_id, client_id.as_str());
    cache
        .set(&blacklist_key, "revoked".to_string(), Some(3600))
        .await;

    tracing::info!(
        "Authorization revoked for user {} client {}",
        user_id,
        client_id.as_str()
    );

    Ok(HttpResponse::NoContent().finish())
}
