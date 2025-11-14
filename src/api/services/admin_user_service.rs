use actix_web::{HttpMessage, HttpRequest, HttpResponse, web};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::errors::AppError;
use crate::security::{Claims, PasswordManager};
use crate::storage::repository::{Pagination, UserListFilter, UserUpdateFields};
use crate::storage::{SeaOrmBackend, UserRepository};

// ============= 请求/响应结构体 =============

#[derive(Debug, Deserialize)]
pub struct UserListQuery {
    pub page: Option<u64>,
    pub page_size: Option<u64>,
    pub role: Option<String>,
    pub is_active: Option<bool>,
    pub keyword: Option<String>,
    pub exclude_deleted: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct UserListResponse {
    pub users: Vec<UserInfo>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
}

#[derive(Debug, Serialize)]
pub struct UserInfo {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub role: String,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
    pub last_login_at: Option<String>,
    pub login_count: i64,
    pub deleted_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateRoleRequest {
    pub role: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateStatusRequest {
    pub is_active: bool,
}

#[derive(Debug, Deserialize)]
pub struct DeleteUserRequest {
    pub permanent: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct ResetPasswordResponse {
    pub new_password: String,
}

#[derive(Debug, Serialize)]
pub struct UserStatsResponse {
    pub total: u64,
    pub active: u64,
    pub inactive: u64,
    pub admins: u64,
    pub regular_users: u64,
    pub deleted: u64,
}

// ============= 处理函数 =============

/// GET /api/admin/users
/// 获取用户列表（分页、筛选）
pub async fn list_users(
    _req: HttpRequest,
    query: web::Query<UserListQuery>,
    storage: web::Data<Arc<SeaOrmBackend>>,
) -> Result<HttpResponse, AppError> {
    // 构建筛选器
    let filter = UserListFilter {
        role: query.role.clone(),
        is_active: query.is_active,
        keyword: query.keyword.clone(),
        created_from: None,
        created_to: None,
        exclude_deleted: query.exclude_deleted.unwrap_or(true),
    };

    // 构建分页参数
    let pagination = Pagination {
        page: query.page.unwrap_or(1),
        page_size: query.page_size.unwrap_or(20).min(100), // 最大 100 条
    };

    // 查询
    let result = storage.list_users(filter, pagination).await?;

    // 转换为响应格式
    let users: Vec<UserInfo> = result
        .users
        .into_iter()
        .map(|user| UserInfo {
            id: user.id,
            username: user.username,
            email: user.email,
            role: user.role,
            is_active: user.is_active,
            created_at: user.created_at.to_rfc3339(),
            updated_at: user.updated_at.to_rfc3339(),
            last_login_at: user.last_login_at.map(|dt| dt.to_rfc3339()),
            login_count: user.login_count,
            deleted_at: user.deleted_at.map(|dt| dt.to_rfc3339()),
        })
        .collect();

    Ok(HttpResponse::Ok().json(UserListResponse {
        users,
        total: result.total,
        page: result.page,
        page_size: result.page_size,
    }))
}

/// GET /api/admin/users/{id}
/// 获取用户详情
pub async fn get_user(
    _req: HttpRequest,
    user_id: web::Path<i64>,
    storage: web::Data<Arc<SeaOrmBackend>>,
) -> Result<HttpResponse, AppError> {
    let user = storage
        .find_by_id(*user_id)
        .await?
        .ok_or(AppError::NotFound)?;

    let user_info = UserInfo {
        id: user.id,
        username: user.username,
        email: user.email,
        role: user.role,
        is_active: user.is_active,
        created_at: user.created_at.to_rfc3339(),
        updated_at: user.updated_at.to_rfc3339(),
        last_login_at: user.last_login_at.map(|dt| dt.to_rfc3339()),
        login_count: user.login_count,
        deleted_at: user.deleted_at.map(|dt| dt.to_rfc3339()),
    };

    Ok(HttpResponse::Ok().json(user_info))
}

/// GET /api/admin/users/stats
/// 获取用户统计
pub async fn get_user_stats(
    _req: HttpRequest,
    storage: web::Data<Arc<SeaOrmBackend>>,
) -> Result<HttpResponse, AppError> {
    let stats = storage.count_users(UserListFilter::default()).await?;

    Ok(HttpResponse::Ok().json(UserStatsResponse {
        total: stats.total,
        active: stats.active,
        inactive: stats.inactive,
        admins: stats.admins,
        regular_users: stats.regular_users,
        deleted: stats.deleted,
    }))
}

/// PATCH /api/admin/users/{id}/role
/// 修改用户角色
pub async fn update_role(
    req: HttpRequest,
    user_id: web::Path<i64>,
    body: web::Json<UpdateRoleRequest>,
    storage: web::Data<Arc<SeaOrmBackend>>,
) -> Result<HttpResponse, AppError> {
    // 获取当前管理员信息
    let claims = req
        .extensions()
        .get::<Claims>()
        .cloned()
        .ok_or(AppError::Unauthorized)?;

    let admin_id: i64 = claims
        .sub
        .parse()
        .map_err(|_| AppError::Internal("Invalid user_id in token".into()))?;

    // 验证角色值
    if body.role != "user" && body.role != "admin" {
        return Err(AppError::BadRequest(
            "Invalid role. Must be 'user' or 'admin'".into(),
        ));
    }

    // 防止管理员修改自己的角色
    if *user_id == admin_id {
        return Err(AppError::BadRequest("Cannot modify your own role".into()));
    }

    // 如果是将管理员降级为普通用户，检查是否是最后一个管理员
    if body.role == "user" {
        let target_user = storage
            .find_by_id(*user_id)
            .await?
            .ok_or(AppError::NotFound)?;

        if target_user.role == "admin" {
            let stats = storage.count_users(UserListFilter::default()).await?;
            if stats.admins <= 1 {
                return Err(AppError::BadRequest(
                    "Cannot remove the last admin user".into(),
                ));
            }
        }
    }

    // 更新角色
    let updated_user = storage.update_role(*user_id, &body.role).await?;

    tracing::info!(
        "User {} role updated to {} by admin {}",
        user_id,
        body.role,
        admin_id
    );

    Ok(HttpResponse::Ok().json(UserInfo {
        id: updated_user.id,
        username: updated_user.username,
        email: updated_user.email,
        role: updated_user.role,
        is_active: updated_user.is_active,
        created_at: updated_user.created_at.to_rfc3339(),
        updated_at: updated_user.updated_at.to_rfc3339(),
        last_login_at: updated_user.last_login_at.map(|dt| dt.to_rfc3339()),
        login_count: updated_user.login_count,
        deleted_at: updated_user.deleted_at.map(|dt| dt.to_rfc3339()),
    }))
}

/// POST /api/admin/users/{id}/reset-password
/// 重置用户密码（生成随机密码）
pub async fn reset_password(
    req: HttpRequest,
    user_id: web::Path<i64>,
    storage: web::Data<Arc<SeaOrmBackend>>,
) -> Result<HttpResponse, AppError> {
    // 获取当前管理员信息
    let claims = req
        .extensions()
        .get::<Claims>()
        .cloned()
        .ok_or(AppError::Unauthorized)?;

    let admin_id: i64 = claims
        .sub
        .parse()
        .map_err(|_| AppError::Internal("Invalid user_id in token".into()))?;

    // 检查用户是否存在
    storage
        .find_by_id(*user_id)
        .await?
        .ok_or(AppError::NotFound)?;

    // 生成 16 位随机密码
    let new_password = generate_random_password(16);
    let password_hash = PasswordManager::hash_password(&new_password)?;

    // 更新密码
    let fields = UserUpdateFields {
        password_hash: Some(password_hash),
        ..Default::default()
    };
    storage.update_user(*user_id, fields).await?;

    tracing::info!("Password reset for user {} by admin {}", user_id, admin_id);

    Ok(HttpResponse::Ok().json(ResetPasswordResponse { new_password }))
}

/// PATCH /api/admin/users/{id}/status
/// 启用/禁用用户
pub async fn update_status(
    req: HttpRequest,
    user_id: web::Path<i64>,
    body: web::Json<UpdateStatusRequest>,
    storage: web::Data<Arc<SeaOrmBackend>>,
) -> Result<HttpResponse, AppError> {
    // 获取当前管理员信息
    let claims = req
        .extensions()
        .get::<Claims>()
        .cloned()
        .ok_or(AppError::Unauthorized)?;

    let admin_id: i64 = claims
        .sub
        .parse()
        .map_err(|_| AppError::Internal("Invalid user_id in token".into()))?;

    // 防止管理员禁用自己
    if *user_id == admin_id {
        return Err(AppError::BadRequest("Cannot modify your own status".into()));
    }

    // 更新状态
    if body.is_active {
        storage.enable_user(*user_id).await?;
        tracing::info!("User {} enabled by admin {}", user_id, admin_id);
    } else {
        storage.disable_user(*user_id).await?;
        tracing::info!("User {} disabled by admin {}", user_id, admin_id);
    }

    // 返回更新后的用户信息
    let user = storage
        .find_by_id(*user_id)
        .await?
        .ok_or(AppError::NotFound)?;

    Ok(HttpResponse::Ok().json(UserInfo {
        id: user.id,
        username: user.username,
        email: user.email,
        role: user.role,
        is_active: user.is_active,
        created_at: user.created_at.to_rfc3339(),
        updated_at: user.updated_at.to_rfc3339(),
        last_login_at: user.last_login_at.map(|dt| dt.to_rfc3339()),
        login_count: user.login_count,
        deleted_at: user.deleted_at.map(|dt| dt.to_rfc3339()),
    }))
}

/// DELETE /api/admin/users/{id}
/// 删除用户（支持软删除和硬删除）
pub async fn delete_user(
    req: HttpRequest,
    user_id: web::Path<i64>,
    query: web::Query<DeleteUserRequest>,
    storage: web::Data<Arc<SeaOrmBackend>>,
) -> Result<HttpResponse, AppError> {
    // 获取当前管理员信息
    let claims = req
        .extensions()
        .get::<Claims>()
        .cloned()
        .ok_or(AppError::Unauthorized)?;

    let admin_id: i64 = claims
        .sub
        .parse()
        .map_err(|_| AppError::Internal("Invalid user_id in token".into()))?;

    // 防止管理员删除自己
    if *user_id == admin_id {
        return Err(AppError::BadRequest("Cannot delete yourself".into()));
    }

    // 检查是否是最后一个管理员
    let user = storage
        .find_by_id(*user_id)
        .await?
        .ok_or(AppError::NotFound)?;

    if user.role == "admin" {
        let stats = storage.count_users(UserListFilter::default()).await?;
        if stats.admins <= 1 {
            return Err(AppError::BadRequest(
                "Cannot delete the last admin user".into(),
            ));
        }
    }

    // 删除用户
    let permanent = query.permanent.unwrap_or(false);
    if permanent {
        // 硬删除
        storage.hard_delete(*user_id).await?;
        tracing::info!("User {} permanently deleted by admin {}", user_id, admin_id);
    } else {
        // 软删除
        storage.soft_delete(*user_id).await?;
        tracing::info!("User {} soft deleted by admin {}", user_id, admin_id);
    }

    Ok(HttpResponse::NoContent().finish())
}

// ============= 辅助函数 =============

/// 生成随机密码
fn generate_random_password(length: usize) -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789\
                            !@#$%^&*";
    let mut rng = rand::rng();

    (0..length)
        .map(|_| {
            let idx = rng.random_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}
