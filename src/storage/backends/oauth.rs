use async_trait::async_trait;
use chrono::Utc;
use sea_orm::*;

use crate::errors::AppError;
use crate::storage::entities::{prelude::*, *};
use crate::storage::repository::{ClientRepository, TokenRepository};

use super::super::backend::SeaOrmBackend;

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
        user_id: i64,
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
        user_id: i64,
        scopes: &str,
        expires_at: chrono::DateTime<Utc>,
    ) -> Result<i64, AppError> {
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
        access_token_id: i64,
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
