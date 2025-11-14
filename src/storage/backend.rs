use async_trait::async_trait;
use chrono::Utc;
use sea_orm::*;
use std::sync::Arc;

use super::entities::{prelude::*, *};
use super::repository::*;
use crate::errors::AppError;

/// SeaORM 存储后端
pub struct SeaOrmBackend {
    db: Arc<DatabaseConnection>,
}

impl SeaOrmBackend {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl UserRepository for SeaOrmBackend {
    async fn create(
        &self,
        username: &str,
        email: &str,
        password_hash: &str,
    ) -> Result<users::Model, AppError> {
        let now = Utc::now();
        let user = users::ActiveModel {
            username: Set(username.to_string()),
            email: Set(email.to_string()),
            password_hash: Set(password_hash.to_string()),
            created_at: Set(now.into()),
            updated_at: Set(now.into()),
            ..Default::default()
        };

        let result = user.insert(self.db.as_ref()).await?;
        Ok(result)
    }

    async fn find_by_id(&self, id: i32) -> Result<Option<users::Model>, AppError> {
        let user = Users::find_by_id(id).one(self.db.as_ref()).await?;
        Ok(user)
    }

    async fn find_by_username(&self, username: &str) -> Result<Option<users::Model>, AppError> {
        let user = Users::find()
            .filter(users::Column::Username.eq(username))
            .one(self.db.as_ref())
            .await?;
        Ok(user)
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<users::Model>, AppError> {
        let user = Users::find()
            .filter(users::Column::Email.eq(email))
            .one(self.db.as_ref())
            .await?;
        Ok(user)
    }
}

#[async_trait]
impl ClientRepository for SeaOrmBackend {
    async fn find_by_client_id(
        &self,
        client_id: &str,
    ) -> Result<Option<o_auth_clients::Model>, AppError> {
        let client = OAuthClients::find()
            .filter(o_auth_clients::Column::ClientId.eq(client_id))
            .one(self.db.as_ref())
            .await?;
        Ok(client)
    }

    async fn verify_redirect_uri(
        &self,
        client_id: &str,
        redirect_uri: &str,
    ) -> Result<bool, AppError> {
        let client = self.find_by_client_id(client_id).await?;

        if let Some(client) = client {
            // redirect_uris 是 JSON array 字符串，解析后验证
            let uris: Vec<String> = serde_json::from_str(&client.redirect_uris).unwrap_or_default();
            Ok(uris.contains(&redirect_uri.to_string()))
        } else {
            Ok(false)
        }
    }
}

#[async_trait]
impl TokenRepository for SeaOrmBackend {
    async fn save_auth_code(
        &self,
        code: &str,
        client_id: &str,
        user_id: i32,
        redirect_uri: &str,
        scopes: &str,
        expires_at: chrono::DateTime<Utc>,
    ) -> Result<(), AppError> {
        let auth_code = authorization_codes::ActiveModel {
            code: Set(code.to_string()),
            client_id: Set(client_id.to_string()),
            user_id: Set(user_id),
            redirect_uri: Set(redirect_uri.to_string()),
            scopes: Set(scopes.to_string()),
            expires_at: Set(expires_at.into()),
            used: Set(false),
            created_at: Set(Utc::now().into()),
            ..Default::default()
        };

        auth_code.insert(self.db.as_ref()).await?;
        Ok(())
    }

    async fn consume_auth_code(
        &self,
        code: &str,
    ) -> Result<Option<authorization_codes::Model>, AppError> {
        let auth_code = AuthorizationCodes::find()
            .filter(authorization_codes::Column::Code.eq(code))
            .filter(authorization_codes::Column::Used.eq(false))
            .one(self.db.as_ref())
            .await?;

        if let Some(ref ac) = auth_code {
            // 标记为已使用
            let mut active: authorization_codes::ActiveModel = ac.clone().into();
            active.used = Set(true);
            active.update(self.db.as_ref()).await?;
        }

        Ok(auth_code)
    }

    async fn save_access_token(
        &self,
        token: &str,
        client_id: &str,
        user_id: i32,
        scopes: &str,
        expires_at: chrono::DateTime<Utc>,
    ) -> Result<i32, AppError> {
        let access_token = access_tokens::ActiveModel {
            token: Set(token.to_string()),
            token_type: Set("Bearer".to_string()),
            client_id: Set(client_id.to_string()),
            user_id: Set(user_id),
            scopes: Set(scopes.to_string()),
            expires_at: Set(expires_at.into()),
            created_at: Set(Utc::now().into()),
            ..Default::default()
        };

        let result = access_token.insert(self.db.as_ref()).await?;
        Ok(result.id)
    }

    async fn save_refresh_token(
        &self,
        token: &str,
        access_token_id: i32,
        expires_at: chrono::DateTime<Utc>,
    ) -> Result<(), AppError> {
        let refresh_token = refresh_tokens::ActiveModel {
            token: Set(token.to_string()),
            access_token_id: Set(access_token_id),
            expires_at: Set(expires_at.into()),
            created_at: Set(Utc::now().into()),
            ..Default::default()
        };

        refresh_token.insert(self.db.as_ref()).await?;
        Ok(())
    }
}
