use chrono::Utc;
use sea_orm::*;

use crate::errors::AppError;
use crate::storage::entities::app_settings;

use super::super::backend::SeaOrmBackend;

// 配置管理方法
impl SeaOrmBackend {
    /// 获取配置项
    pub async fn get_setting(
        &self,
        key: &str,
    ) -> Result<Option<(String, Option<String>, Option<i64>, Option<bool>)>, AppError> {
        let setting = app_settings::Entity::find()
            .filter(app_settings::Column::Key.eq(key))
            .one(self.db.as_ref())
            .await?;

        if let Some(s) = setting {
            Ok(Some((
                s.value_type,
                s.value_string,
                s.value_int,
                s.value_bool,
            )))
        } else {
            Ok(None)
        }
    }

    /// 设置配置项
    pub async fn set_setting(
        &self,
        key: &str,
        value_type: &str,
        value_string: Option<&str>,
        value_int: Option<i64>,
        value_bool: Option<bool>,
        updated_by: Option<i64>,
    ) -> Result<(), AppError> {
        // 查找是否存在
        let existing = app_settings::Entity::find()
            .filter(app_settings::Column::Key.eq(key))
            .one(self.db.as_ref())
            .await?;

        if let Some(existing) = existing {
            // 更新
            let mut active: app_settings::ActiveModel = existing.into();
            active.value_type = Set(value_type.to_string());
            active.value_string = Set(value_string.map(|s| s.to_string()));
            active.value_int = Set(value_int);
            active.value_bool = Set(value_bool);
            active.updated_at = Set(Utc::now().into());
            active.updated_by = Set(updated_by);
            active.update(self.db.as_ref()).await?;
        } else {
            // 插入
            let setting = app_settings::ActiveModel {
                key: Set(key.to_string()),
                value_type: Set(value_type.to_string()),
                value_string: Set(value_string.map(|s| s.to_string())),
                value_int: Set(value_int),
                value_bool: Set(value_bool),
                updated_at: Set(Utc::now().into()),
                updated_by: Set(updated_by),
                ..Default::default()
            };
            setting.insert(self.db.as_ref()).await?;
        }

        Ok(())
    }

    /// 获取注册配置
    pub async fn get_registration_config(
        &self,
    ) -> Result<crate::config::RegistrationConfig, AppError> {
        use crate::config::RegistrationConfig;

        const CACHE_KEY: &str = "config:registration";
        const CACHE_TTL: u64 = 300; // 5 分钟

        // 1. 尝试从缓存读取
        if let Some(cache) = &self.cache
            && let Some(cached) = cache.get(CACHE_KEY).await
            && let Ok(config) = serde_json::from_str::<RegistrationConfig>(&cached)
        {
            tracing::debug!("Registration config loaded from cache");
            return Ok(config);
        }

        // 2. 从数据库读取
        let mut config = RegistrationConfig::default();

        // 读取各个配置项
        if let Some((_, _, _, Some(v))) = self.get_setting("allow_registration").await? {
            config.allow_registration = v;
        }

        if let Some((_, Some(v), _, _)) = self.get_setting("allowed_email_domains").await?
            && !v.is_empty()
        {
            config.allowed_email_domains = v.split(',').map(|s| s.trim().to_string()).collect();
        }

        if let Some((_, _, Some(v), _)) = self.get_setting("min_username_length").await? {
            config.min_username_length = v as u32;
        }

        if let Some((_, _, Some(v), _)) = self.get_setting("max_username_length").await? {
            config.max_username_length = v as u32;
        }

        if let Some((_, _, Some(v), _)) = self.get_setting("min_password_length").await? {
            config.min_password_length = v as u32;
        }

        if let Some((_, _, _, Some(v))) = self.get_setting("password_require_uppercase").await? {
            config.password_require_uppercase = v;
        }

        if let Some((_, _, _, Some(v))) = self.get_setting("password_require_lowercase").await? {
            config.password_require_lowercase = v;
        }

        if let Some((_, _, _, Some(v))) = self.get_setting("password_require_numbers").await? {
            config.password_require_numbers = v;
        }

        if let Some((_, _, _, Some(v))) = self.get_setting("password_require_special").await? {
            config.password_require_special = v;
        }

        if let Some((_, _, _, Some(v))) = self.get_setting("require_invite_code").await? {
            config.require_invite_code = v;
        }

        // 3. 写入缓存
        if let Some(cache) = &self.cache
            && let Ok(json) = serde_json::to_string(&config)
        {
            cache.set(CACHE_KEY, json, Some(CACHE_TTL)).await;
            tracing::debug!("Registration config cached");
        }

        Ok(config)
    }

    /// 更新注册配置
    pub async fn update_registration_config(
        &self,
        config: &crate::config::RegistrationConfig,
        updated_by: i64,
    ) -> Result<(), AppError> {
        // 获取旧配置用于审计日志
        let old_config = self.get_registration_config().await?;

        self.set_setting(
            "allow_registration",
            "bool",
            None,
            None,
            Some(config.allow_registration),
            Some(updated_by),
        )
        .await?;

        let domains = config.allowed_email_domains.join(",");
        self.set_setting(
            "allowed_email_domains",
            "string",
            Some(&domains),
            None,
            None,
            Some(updated_by),
        )
        .await?;

        self.set_setting(
            "min_username_length",
            "int",
            None,
            Some(config.min_username_length as i64),
            None,
            Some(updated_by),
        )
        .await?;
        self.set_setting(
            "max_username_length",
            "int",
            None,
            Some(config.max_username_length as i64),
            None,
            Some(updated_by),
        )
        .await?;
        self.set_setting(
            "min_password_length",
            "int",
            None,
            Some(config.min_password_length as i64),
            None,
            Some(updated_by),
        )
        .await?;

        self.set_setting(
            "password_require_uppercase",
            "bool",
            None,
            None,
            Some(config.password_require_uppercase),
            Some(updated_by),
        )
        .await?;
        self.set_setting(
            "password_require_lowercase",
            "bool",
            None,
            None,
            Some(config.password_require_lowercase),
            Some(updated_by),
        )
        .await?;
        self.set_setting(
            "password_require_numbers",
            "bool",
            None,
            None,
            Some(config.password_require_numbers),
            Some(updated_by),
        )
        .await?;
        self.set_setting(
            "password_require_special",
            "bool",
            None,
            None,
            Some(config.password_require_special),
            Some(updated_by),
        )
        .await?;

        self.set_setting(
            "require_invite_code",
            "bool",
            None,
            None,
            Some(config.require_invite_code),
            Some(updated_by),
        )
        .await?;

        // 记录审计日志
        let old_json = serde_json::to_string(&old_config).unwrap_or_default();
        let new_json = serde_json::to_string(&config).unwrap_or_default();
        self.log_config_change(
            "registration_config",
            Some(old_json),
            Some(new_json),
            updated_by,
            "update",
        )
        .await?;

        // 清除缓存
        if let Some(cache) = &self.cache {
            cache.delete("config:registration").await;
            tracing::debug!("Registration config cache invalidated");
        }

        Ok(())
    }

    /// 获取认证策略配置
    pub async fn get_auth_policy_config(
        &self,
    ) -> Result<crate::config::AuthPolicyConfig, AppError> {
        use crate::config::AuthPolicyConfig;

        const CACHE_KEY: &str = "config:auth_policy";
        const CACHE_TTL: u64 = 300; // 5 分钟

        // 1. 尝试从缓存读取
        if let Some(cache) = &self.cache
            && let Some(cached) = cache.get(CACHE_KEY).await
            && let Ok(config) = serde_json::from_str::<AuthPolicyConfig>(&cached)
        {
            tracing::debug!("Auth policy config loaded from cache");
            return Ok(config);
        }

        // 2. 从数据库读取
        let mut config = AuthPolicyConfig::default();

        if let Some((_, _, Some(v), _)) = self.get_setting("access_token_expire").await? {
            config.access_token_expire = v;
        }
        if let Some((_, _, Some(v), _)) = self.get_setting("refresh_token_expire").await? {
            config.refresh_token_expire = v;
        }
        if let Some((_, _, Some(v), _)) = self.get_setting("authorization_code_expire").await? {
            config.authorization_code_expire = v;
        }

        // 3. 写入缓存
        if let Some(cache) = &self.cache
            && let Ok(json) = serde_json::to_string(&config)
        {
            cache.set(CACHE_KEY, json, Some(CACHE_TTL)).await;
            tracing::debug!("Auth policy config cached");
        }

        Ok(config)
    }

    /// 更新认证策略配置
    pub async fn update_auth_policy_config(
        &self,
        config: &crate::config::AuthPolicyConfig,
        updated_by: i64,
    ) -> Result<(), AppError> {
        // 获取旧配置用于审计日志
        let old_config = self.get_auth_policy_config().await?;

        self.set_setting(
            "access_token_expire",
            "int",
            None,
            Some(config.access_token_expire),
            None,
            Some(updated_by),
        )
        .await?;

        self.set_setting(
            "refresh_token_expire",
            "int",
            None,
            Some(config.refresh_token_expire),
            None,
            Some(updated_by),
        )
        .await?;

        self.set_setting(
            "authorization_code_expire",
            "int",
            None,
            Some(config.authorization_code_expire),
            None,
            Some(updated_by),
        )
        .await?;

        // 记录审计日志
        let old_json = serde_json::to_string(&old_config).unwrap_or_default();
        let new_json = serde_json::to_string(&config).unwrap_or_default();
        self.log_config_change(
            "auth_policy_config",
            Some(old_json),
            Some(new_json),
            updated_by,
            "update",
        )
        .await?;

        // 清除缓存
        if let Some(cache) = &self.cache {
            cache.delete("config:auth_policy").await;
            tracing::debug!("Auth policy config cache invalidated");
        }

        Ok(())
    }

    /// 获取缓存策略配置
    pub async fn get_cache_policy_config(
        &self,
    ) -> Result<crate::config::CachePolicyConfig, AppError> {
        use crate::config::CachePolicyConfig;

        const CACHE_KEY: &str = "config:cache_policy";
        const CACHE_TTL: u64 = 300; // 5 分钟

        // 1. 尝试从缓存读取
        if let Some(cache) = &self.cache
            && let Some(cached) = cache.get(CACHE_KEY).await
            && let Ok(config) = serde_json::from_str::<CachePolicyConfig>(&cached)
        {
            tracing::debug!("Cache policy config loaded from cache");
            return Ok(config);
        }

        // 2. 从数据库读取
        let mut config = CachePolicyConfig::default();

        if let Some((_, _, Some(v), _)) = self.get_setting("default_ttl").await? {
            config.default_ttl = v;
        }

        // 3. 写入缓存
        if let Some(cache) = &self.cache
            && let Ok(json) = serde_json::to_string(&config)
        {
            cache.set(CACHE_KEY, json, Some(CACHE_TTL)).await;
            tracing::debug!("Cache policy config cached");
        }

        Ok(config)
    }

    /// 更新缓存策略配置
    pub async fn update_cache_policy_config(
        &self,
        config: &crate::config::CachePolicyConfig,
        updated_by: i64,
    ) -> Result<(), AppError> {
        // 获取旧配置用于审计日志
        let old_config = self.get_cache_policy_config().await?;

        self.set_setting(
            "default_ttl",
            "int",
            None,
            Some(config.default_ttl),
            None,
            Some(updated_by),
        )
        .await?;

        // 记录审计日志
        let old_json = serde_json::to_string(&old_config).unwrap_or_default();
        let new_json = serde_json::to_string(&config).unwrap_or_default();
        self.log_config_change(
            "cache_policy_config",
            Some(old_json),
            Some(new_json),
            updated_by,
            "update",
        )
        .await?;

        // 清除缓存
        if let Some(cache) = &self.cache {
            cache.delete("config:cache_policy").await;
            tracing::debug!("Cache policy config cache invalidated");
        }

        Ok(())
    }
}
