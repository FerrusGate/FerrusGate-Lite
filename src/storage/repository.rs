use async_trait::async_trait;
use chrono::Utc;

use super::entities::{authorization_codes, o_auth_clients, users};
use crate::errors::AppError;

/// 用户仓储
#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn create(
        &self,
        username: &str,
        email: &str,
        password_hash: &str,
    ) -> Result<users::Model, AppError>;
    async fn find_by_id(&self, id: i32) -> Result<Option<users::Model>, AppError>;
    async fn find_by_username(&self, username: &str) -> Result<Option<users::Model>, AppError>;
    async fn find_by_email(&self, email: &str) -> Result<Option<users::Model>, AppError>;
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
        user_id: i32,
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
        user_id: i32,
        scopes: &str,
        expires_at: chrono::DateTime<Utc>,
    ) -> Result<i32, AppError>;

    async fn save_refresh_token(
        &self,
        token: &str,
        access_token_id: i32,
        expires_at: chrono::DateTime<Utc>,
    ) -> Result<(), AppError>;
}
