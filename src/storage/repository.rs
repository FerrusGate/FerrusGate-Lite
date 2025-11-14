use async_trait::async_trait;
use chrono::Utc;
use serde::{Deserialize, Serialize};

use super::entities::{authorization_codes, o_auth_clients, users};
use crate::errors::AppError;

/// 用户列表查询过滤器
#[derive(Debug, Clone, Default, Deserialize)]
pub struct UserListFilter {
    /// 按角色筛选
    pub role: Option<String>,
    /// 按状态筛选 (is_active)
    pub is_active: Option<bool>,
    /// 按关键词搜索 (username 或 email)
    pub keyword: Option<String>,
    /// 注册时间范围 - 开始
    pub created_from: Option<chrono::DateTime<Utc>>,
    /// 注册时间范围 - 结束
    pub created_to: Option<chrono::DateTime<Utc>>,
    /// 是否只显示未删除的用户
    pub exclude_deleted: bool,
}

/// 分页参数
#[derive(Debug, Clone, Deserialize)]
pub struct Pagination {
    pub page: u64,
    pub page_size: u64,
}

impl Default for Pagination {
    fn default() -> Self {
        Self {
            page: 1,
            page_size: 20,
        }
    }
}

/// 用户列表返回结果
#[derive(Debug, Clone, Serialize)]
pub struct UserListResult {
    pub users: Vec<users::Model>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
}

/// 用户更新字段（部分更新）
#[derive(Debug, Clone, Default)]
pub struct UserUpdateFields {
    pub username: Option<String>,
    pub email: Option<String>,
    pub password_hash: Option<String>,
    pub role: Option<String>,
    pub is_active: Option<bool>,
}

/// 用户统计数据
#[derive(Debug, Clone, Serialize)]
pub struct UserStats {
    pub total: u64,
    pub active: u64,
    pub inactive: u64,
    pub admins: u64,
    pub regular_users: u64,
    pub deleted: u64,
}

/// 用户仓储
#[async_trait]
pub trait UserRepository: Send + Sync {
    // ===== 基础 CRUD =====
    async fn create(
        &self,
        username: &str,
        email: &str,
        password_hash: &str,
    ) -> Result<users::Model, AppError>;
    async fn find_by_id(&self, id: i64) -> Result<Option<users::Model>, AppError>;
    async fn find_by_username(&self, username: &str) -> Result<Option<users::Model>, AppError>;
    async fn find_by_email(&self, email: &str) -> Result<Option<users::Model>, AppError>;

    // ===== 用户管理功能 =====
    /// 分页查询用户列表（支持筛选）
    async fn list_users(
        &self,
        filter: UserListFilter,
        pagination: Pagination,
    ) -> Result<UserListResult, AppError>;

    /// 更新用户信息
    async fn update_user(
        &self,
        id: i64,
        fields: UserUpdateFields,
    ) -> Result<users::Model, AppError>;

    /// 修改用户角色
    async fn update_role(&self, id: i64, role: &str) -> Result<users::Model, AppError>;

    /// 软删除用户（设置 deleted_at）
    async fn soft_delete(&self, id: i64) -> Result<(), AppError>;

    /// 硬删除用户（从数据库移除）
    async fn hard_delete(&self, id: i64) -> Result<(), AppError>;

    /// 禁用用户
    async fn disable_user(&self, id: i64) -> Result<(), AppError>;

    /// 启用用户
    async fn enable_user(&self, id: i64) -> Result<(), AppError>;

    /// 统计用户数据
    async fn count_users(&self, filter: UserListFilter) -> Result<UserStats, AppError>;

    /// 更新用户登录信息
    async fn update_login_info(&self, id: i64) -> Result<(), AppError>;
}

/// OAuth 客户端仓储
#[async_trait]
pub trait ClientRepository: Send + Sync {
    async fn find_by_client_id(
        &self,
        client_id: &str,
    ) -> Result<Option<o_auth_clients::Model>, AppError>;
    async fn verify_redirect_uri(
        &self,
        client_id: &str,
        redirect_uri: &str,
    ) -> Result<bool, AppError>;
}

/// Token 仓储
#[async_trait]
pub trait TokenRepository: Send + Sync {
    async fn save_auth_code(
        &self,
        code: &str,
        client_id: &str,
        user_id: i64,
        redirect_uri: &str,
        scopes: &str,
        expires_at: chrono::DateTime<Utc>,
    ) -> Result<(), AppError>;

    async fn consume_auth_code(
        &self,
        code: &str,
    ) -> Result<Option<authorization_codes::Model>, AppError>;

    async fn save_access_token(
        &self,
        token: &str,
        client_id: &str,
        user_id: i64,
        scopes: &str,
        expires_at: chrono::DateTime<Utc>,
    ) -> Result<i64, AppError>;

    async fn save_refresh_token(
        &self,
        token: &str,
        access_token_id: i64,
        expires_at: chrono::DateTime<Utc>,
    ) -> Result<(), AppError>;
}
