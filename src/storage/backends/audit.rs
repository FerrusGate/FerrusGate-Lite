use chrono::Utc;
use sea_orm::*;

use crate::errors::AppError;
use crate::storage::entities::config_audit_logs;

use super::super::backend::SeaOrmBackend;

// 配置审计日志方法
impl SeaOrmBackend {
    /// 记录配置变更
    pub async fn log_config_change(
        &self,
        config_key: &str,
        old_value: Option<String>,
        new_value: Option<String>,
        changed_by: i64,
        change_type: &str,
    ) -> Result<(), AppError> {
        let log = config_audit_logs::ActiveModel {
            config_key: Set(config_key.to_string()),
            old_value: Set(old_value),
            new_value: Set(new_value),
            changed_by: Set(changed_by),
            changed_at: Set(Utc::now().into()),
            change_type: Set(Some(change_type.to_string())),
            ..Default::default()
        };

        log.insert(self.db.as_ref()).await?;
        Ok(())
    }

    /// 获取配置变更历史
    pub async fn get_config_audit_logs(
        &self,
        limit: Option<u64>,
    ) -> Result<Vec<config_audit_logs::Model>, AppError> {
        let mut query =
            config_audit_logs::Entity::find().order_by_desc(config_audit_logs::Column::ChangedAt);

        if let Some(limit) = limit {
            query = query.limit(limit);
        }

        let logs = query.all(self.db.as_ref()).await?;
        Ok(logs)
    }

    /// 获取特定配置键的变更历史
    pub async fn get_config_audit_logs_by_key(
        &self,
        config_key: &str,
        limit: Option<u64>,
    ) -> Result<Vec<config_audit_logs::Model>, AppError> {
        let mut query = config_audit_logs::Entity::find()
            .filter(config_audit_logs::Column::ConfigKey.eq(config_key))
            .order_by_desc(config_audit_logs::Column::ChangedAt);

        if let Some(limit) = limit {
            query = query.limit(limit);
        }

        let logs = query.all(self.db.as_ref()).await?;
        Ok(logs)
    }
}
