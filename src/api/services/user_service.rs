use actix_web::{web, HttpResponse, HttpRequest, HttpMessage};
use serde::Serialize;
use std::sync::Arc;

use crate::errors::AppError;
use crate::security::Claims;
use crate::storage::{UserRepository, SeaOrmBackend};

#[derive(Debug, Serialize)]
pub struct UserProfileResponse {
    pub id: i32,
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
    let user_id: i32 = claims
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
    _storage: web::Data<Arc<SeaOrmBackend>>,
) -> Result<HttpResponse, AppError> {
    // 从请求扩展中提取 Claims
    let claims = req
        .extensions()
        .get::<Claims>()
        .cloned()
        .ok_or(AppError::Unauthorized)?;

    let _user_id: i32 = claims
        .sub
        .parse()
        .map_err(|_| AppError::Internal("Invalid user_id in token".into()))?;

    // TODO: 实际应该从数据库查询用户的授权记录
    // 这里返回空列表作为示例
    let authorizations: Vec<AuthorizationInfo> = vec![];

    Ok(HttpResponse::Ok().json(authorizations))
}

/// DELETE /api/user/authorizations/{client_id}
/// 撤销对某个应用的授权
pub async fn revoke_authorization(
    req: HttpRequest,
    client_id: web::Path<String>,
    _cache: web::Data<Arc<crate::cache::CompositeCache>>,
) -> Result<HttpResponse, AppError> {
    // 从请求扩展中提取 Claims
    let claims = req
        .extensions()
        .get::<Claims>()
        .cloned()
        .ok_or(AppError::Unauthorized)?;

    let user_id: i32 = claims
        .sub
        .parse()
        .map_err(|_| AppError::Internal("Invalid user_id in token".into()))?;

    // TODO: 实际应该从数据库删除授权记录，并将相关 token 加入黑名单
    // 这里简化处理

    tracing::info!(
        "Authorization revoked for user {} client {}",
        user_id,
        client_id.as_str()
    );

    // 可以将用户的所有 token 加入黑名单
    // cache.set(&format!("blacklist:user:{}", user_id), "revoked", Some(3600)).await;

    Ok(HttpResponse::NoContent().finish())
}
