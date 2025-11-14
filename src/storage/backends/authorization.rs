use chrono::Utc;
use sea_orm::*;

use crate::errors::AppError;
use crate::storage::entities::{access_tokens, o_auth_clients, refresh_tokens};

use super::super::backend::SeaOrmBackend;

/// 用户授权信息
#[derive(Debug, serde::Serialize)]
pub struct UserAuthorizationInfo {
    pub client_id: String,
    pub client_name: String,
    pub scopes: Vec<String>,
    pub granted_at: sea_orm::prelude::DateTimeWithTimeZone,
}

// 用户授权管理方法
impl SeaOrmBackend {
    /// 获取用户的授权列表
    pub async fn get_user_authorizations(
        &self,
        user_id: i64,
    ) -> Result<Vec<UserAuthorizationInfo>, AppError> {
        // 查询用户的所有有效 access_tokens
        let tokens = access_tokens::Entity::find()
            .filter(access_tokens::Column::UserId.eq(user_id))
            .filter(access_tokens::Column::ExpiresAt.gt(Utc::now()))
            .all(self.db.as_ref())
            .await?;

        // 按 client_id 去重并整合信息
        let mut auth_map: std::collections::HashMap<String, UserAuthorizationInfo> =
            std::collections::HashMap::new();

        for token in tokens {
            let entry =
                auth_map
                    .entry(token.client_id.clone())
                    .or_insert_with(|| UserAuthorizationInfo {
                        client_id: token.client_id.clone(),
                        client_name: token.client_id.clone(), // 暂时使用 client_id 作为名称
                        scopes: vec![],
                        granted_at: token.created_at,
                    });

            // 合并 scopes（去重）
            let scopes: Vec<String> = token
                .scopes
                .split_whitespace()
                .map(|s| s.to_string())
                .collect();
            for scope in scopes {
                if !entry.scopes.contains(&scope) {
                    entry.scopes.push(scope);
                }
            }

            // 保留最早的授权时间
            if token.created_at.timestamp() < entry.granted_at.timestamp() {
                entry.granted_at = token.created_at;
            }
        }

        // 查询 OAuth clients 信息来获取友好的名称
        let client_ids: Vec<String> = auth_map.keys().cloned().collect();
        if !client_ids.is_empty() {
            let clients = o_auth_clients::Entity::find()
                .filter(o_auth_clients::Column::ClientId.is_in(client_ids))
                .all(self.db.as_ref())
                .await?;

            // 更新 client_name
            for client in clients {
                if let Some(auth) = auth_map.get_mut(&client.client_id) {
                    auth.client_name = client.name;
                }
            }
        }

        Ok(auth_map.into_values().collect())
    }

    /// 撤销用户对某个应用的授权
    pub async fn revoke_user_authorization(
        &self,
        user_id: i64,
        client_id: &str,
    ) -> Result<(), AppError> {
        // 1. 查找该用户对该应用的所有 access_tokens
        let tokens = access_tokens::Entity::find()
            .filter(access_tokens::Column::UserId.eq(user_id))
            .filter(access_tokens::Column::ClientId.eq(client_id))
            .all(self.db.as_ref())
            .await?;

        // 2. 删除相关的 refresh_tokens 和 access_tokens
        for token in tokens {
            // 删除关联的 refresh_tokens
            refresh_tokens::Entity::delete_many()
                .filter(refresh_tokens::Column::AccessTokenId.eq(token.id))
                .exec(self.db.as_ref())
                .await?;

            // 删除 access_token
            access_tokens::Entity::delete_by_id(token.id)
                .exec(self.db.as_ref())
                .await?;
        }

        Ok(())
    }
}
