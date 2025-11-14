use chrono::Utc;
use sea_orm::*;

use crate::errors::AppError;
use crate::storage::entities::invite_codes;

use super::super::backend::SeaOrmBackend;

/// 邀请码统计信息
#[derive(Debug, serde::Serialize)]
pub struct InviteStats {
    pub total_count: i64,      // 总邀请码数
    pub active_count: i64,     // 可用邀请码数
    pub fully_used_count: i64, // 已用完的邀请码数
    pub expired_count: i64,    // 已过期的邀请码数
    pub total_uses: i64,       // 总使用次数
    pub total_capacity: i64,   // 总容量（所有邀请码的 max_uses 之和）
    pub usage_rate: i32,       // 使用率（百分比）
}

// 邀请码管理方法
impl SeaOrmBackend {
    /// 创建邀请码
    pub async fn create_invite_code(
        &self,
        code: &str,
        created_by: i64,
        max_uses: i32,
        expires_at: Option<chrono::DateTime<Utc>>,
    ) -> Result<invite_codes::Model, AppError> {
        let invite = invite_codes::ActiveModel {
            code: Set(code.to_string()),
            created_by: Set(created_by),
            used_by: Set(None),
            max_uses: Set(max_uses as i64),
            used_count: Set(0),
            expires_at: Set(expires_at.map(|dt| dt.into())),
            created_at: Set(Utc::now().into()),
            ..Default::default()
        };

        let result = invite.insert(self.db.as_ref()).await?;
        Ok(result)
    }

    /// 列出所有邀请码
    pub async fn list_invite_codes(&self) -> Result<Vec<invite_codes::Model>, AppError> {
        let codes = invite_codes::Entity::find()
            .order_by_desc(invite_codes::Column::CreatedAt)
            .all(self.db.as_ref())
            .await?;
        Ok(codes)
    }

    /// 查找邀请码
    pub async fn find_invite_code(
        &self,
        code: &str,
    ) -> Result<Option<invite_codes::Model>, AppError> {
        let invite = invite_codes::Entity::find()
            .filter(invite_codes::Column::Code.eq(code))
            .one(self.db.as_ref())
            .await?;
        Ok(invite)
    }

    /// 验证并使用邀请码
    pub async fn verify_and_use_invite_code(
        &self,
        code: &str,
        user_id: i64,
    ) -> Result<(), AppError> {
        let invite = self
            .find_invite_code(code)
            .await?
            .ok_or_else(|| AppError::BadRequest("Invalid invite code".into()))?;

        // 检查是否过期
        if let Some(expires_at) = invite.expires_at
            && Utc::now().timestamp() > expires_at.timestamp()
        {
            return Err(AppError::BadRequest("Invite code expired".into()));
        }

        // 检查使用次数
        if invite.used_count >= invite.max_uses {
            return Err(AppError::BadRequest(
                "Invite code has been fully used".into(),
            ));
        }

        // 增加使用次数
        let mut active: invite_codes::ActiveModel = invite.into();
        let current_count = match &active.used_count {
            Set(count) => *count,
            _ => 0,
        };
        let new_count = current_count + 1;
        active.used_count = Set(new_count);

        // 如果是第一次使用，记录使用者
        if new_count == 1 {
            active.used_by = Set(Some(user_id));
        }

        active.update(self.db.as_ref()).await?;
        Ok(())
    }

    /// 撤销邀请码(删除)
    pub async fn revoke_invite_code(&self, code: &str) -> Result<(), AppError> {
        let invite = self
            .find_invite_code(code)
            .await?
            .ok_or_else(|| AppError::NotFound)?;

        invite_codes::Entity::delete_by_id(invite.id)
            .exec(self.db.as_ref())
            .await?;

        Ok(())
    }

    /// 获取邀请码统计信息
    pub async fn get_invite_stats(&self) -> Result<InviteStats, AppError> {
        let all_invites = self.list_invite_codes().await?;
        let now = Utc::now();

        let total_count = all_invites.len() as i64;
        let mut active_count = 0i64;
        let mut fully_used_count = 0i64;
        let mut expired_count = 0i64;
        let mut total_uses = 0i64;
        let mut total_capacity = 0i64;

        for invite in &all_invites {
            total_uses += invite.used_count;
            total_capacity += invite.max_uses;

            // 检查是否过期
            let is_expired = invite
                .expires_at
                .as_ref()
                .map(|exp| now.timestamp() > exp.timestamp())
                .unwrap_or(false);

            // 检查是否用完
            let is_fully_used = invite.used_count >= invite.max_uses;

            if is_expired {
                expired_count += 1;
            } else if is_fully_used {
                fully_used_count += 1;
            } else {
                active_count += 1;
            }
        }

        let usage_rate = if total_capacity > 0 {
            (total_uses as f64 / total_capacity as f64 * 100.0).round() as i32
        } else {
            0
        };

        Ok(InviteStats {
            total_count,
            active_count,
            fully_used_count,
            expired_count,
            total_uses,
            total_capacity,
            usage_rate,
        })
    }
}
