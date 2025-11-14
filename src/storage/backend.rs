use async_trait::async_trait;
use chrono::Utc;
use sea_orm::*;
use std::sync::Arc;

use super::entities::{prelude::*, *};
use super::repository::*;
use crate::cache::CompositeCache;
use crate::errors::AppError;

/// SeaORM 存储后端
pub struct SeaOrmBackend {
    db: Arc<DatabaseConnection>,
    cache: Option<Arc<CompositeCache>>,
}

impl SeaOrmBackend {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db, cache: None }
    }

    pub fn with_cache(db: Arc<DatabaseConnection>, cache: Arc<CompositeCache>) -> Self {
        Self {
            db,
            cache: Some(cache),
        }
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

    async fn find_by_id(&self, id: i64) -> Result<Option<users::Model>, AppError> {
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

    async fn list_users(
        &self,
        filter: UserListFilter,
        pagination: Pagination,
    ) -> Result<UserListResult, AppError> {
        // 构建基础查询
        let mut query = Users::find();

        // 应用筛选条件
        if let Some(role) = filter.role {
            query = query.filter(users::Column::Role.eq(role));
        }

        if let Some(is_active) = filter.is_active {
            query = query.filter(users::Column::IsActive.eq(is_active));
        }

        if let Some(keyword) = filter.keyword {
            // 搜索 username 或 email
            let keyword_pattern = format!("%{}%", keyword);
            query = query.filter(
                Condition::any()
                    .add(users::Column::Username.like(&keyword_pattern))
                    .add(users::Column::Email.like(&keyword_pattern)),
            );
        }

        if let Some(created_from) = filter.created_from {
            query = query.filter(users::Column::CreatedAt.gte(created_from));
        }

        if let Some(created_to) = filter.created_to {
            query = query.filter(users::Column::CreatedAt.lte(created_to));
        }

        if filter.exclude_deleted {
            query = query.filter(users::Column::DeletedAt.is_null());
        }

        // 计算总数
        let total = query.clone().count(self.db.as_ref()).await?;

        // 应用分页
        let offset = (pagination.page - 1) * pagination.page_size;
        let user_list = query
            .offset(offset)
            .limit(pagination.page_size)
            .order_by_desc(users::Column::CreatedAt)
            .all(self.db.as_ref())
            .await?;

        Ok(UserListResult {
            users: user_list,
            total,
            page: pagination.page,
            page_size: pagination.page_size,
        })
    }

    async fn update_user(
        &self,
        id: i64,
        fields: UserUpdateFields,
    ) -> Result<users::Model, AppError> {
        // 查找用户
        let user = Users::find_by_id(id)
            .one(self.db.as_ref())
            .await?
            .ok_or(AppError::NotFound)?;

        // 转换为 ActiveModel
        let mut active: users::ActiveModel = user.into();
        active.updated_at = Set(Utc::now().into());

        // 应用更新字段
        if let Some(username) = fields.username {
            active.username = Set(username);
        }
        if let Some(email) = fields.email {
            active.email = Set(email);
        }
        if let Some(password_hash) = fields.password_hash {
            active.password_hash = Set(password_hash);
        }
        if let Some(role) = fields.role {
            active.role = Set(role);
        }
        if let Some(is_active) = fields.is_active {
            active.is_active = Set(is_active);
        }

        // 保存更新
        let updated_user = active.update(self.db.as_ref()).await?;
        Ok(updated_user)
    }

    async fn update_role(&self, id: i64, role: &str) -> Result<users::Model, AppError> {
        let user = Users::find_by_id(id)
            .one(self.db.as_ref())
            .await?
            .ok_or(AppError::NotFound)?;

        let mut active: users::ActiveModel = user.into();
        active.role = Set(role.to_string());
        active.updated_at = Set(Utc::now().into());

        let updated_user = active.update(self.db.as_ref()).await?;
        Ok(updated_user)
    }

    async fn soft_delete(&self, id: i64) -> Result<(), AppError> {
        let user = Users::find_by_id(id)
            .one(self.db.as_ref())
            .await?
            .ok_or(AppError::NotFound)?;

        let mut active: users::ActiveModel = user.into();
        active.deleted_at = Set(Some(Utc::now().into()));
        active.is_active = Set(false); // 同时禁用
        active.updated_at = Set(Utc::now().into());

        active.update(self.db.as_ref()).await?;
        Ok(())
    }

    async fn hard_delete(&self, id: i64) -> Result<(), AppError> {
        Users::delete_by_id(id).exec(self.db.as_ref()).await?;
        Ok(())
    }

    async fn disable_user(&self, id: i64) -> Result<(), AppError> {
        let user = Users::find_by_id(id)
            .one(self.db.as_ref())
            .await?
            .ok_or(AppError::NotFound)?;

        let mut active: users::ActiveModel = user.into();
        active.is_active = Set(false);
        active.updated_at = Set(Utc::now().into());

        active.update(self.db.as_ref()).await?;
        Ok(())
    }

    async fn enable_user(&self, id: i64) -> Result<(), AppError> {
        let user = Users::find_by_id(id)
            .one(self.db.as_ref())
            .await?
            .ok_or(AppError::NotFound)?;

        let mut active: users::ActiveModel = user.into();
        active.is_active = Set(true);
        active.updated_at = Set(Utc::now().into());

        active.update(self.db.as_ref()).await?;
        Ok(())
    }

    async fn count_users(&self, filter: UserListFilter) -> Result<UserStats, AppError> {
        // 总用户数（包含删除的）
        let total = Users::find().count(self.db.as_ref()).await?;

        // 活跃用户
        let active = Users::find()
            .filter(users::Column::IsActive.eq(true))
            .filter(users::Column::DeletedAt.is_null())
            .count(self.db.as_ref())
            .await?;

        // 非活跃用户
        let inactive = Users::find()
            .filter(users::Column::IsActive.eq(false))
            .filter(users::Column::DeletedAt.is_null())
            .count(self.db.as_ref())
            .await?;

        // 管理员数量
        let admins = Users::find()
            .filter(users::Column::Role.eq("admin"))
            .filter(users::Column::DeletedAt.is_null())
            .count(self.db.as_ref())
            .await?;

        // 普通用户数量
        let regular_users = Users::find()
            .filter(users::Column::Role.eq("user"))
            .filter(users::Column::DeletedAt.is_null())
            .count(self.db.as_ref())
            .await?;

        // 已删除用户
        let deleted = Users::find()
            .filter(users::Column::DeletedAt.is_not_null())
            .count(self.db.as_ref())
            .await?;

        Ok(UserStats {
            total,
            active,
            inactive,
            admins,
            regular_users,
            deleted,
        })
    }

    async fn update_login_info(&self, id: i64) -> Result<(), AppError> {
        let user = Users::find_by_id(id)
            .one(self.db.as_ref())
            .await?
            .ok_or(AppError::NotFound)?;

        let mut active: users::ActiveModel = user.into();
        active.last_login_at = Set(Some(Utc::now().into()));

        // 增加登录次数
        let current_count = match &active.login_count {
            Set(count) => *count,
            _ => 0,
        };
        active.login_count = Set(current_count + 1);

        active.update(self.db.as_ref()).await?;
        Ok(())
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

// 配置管理方法
impl SeaOrmBackend {
    /// 获取配置项
    pub async fn get_setting(
        &self,
        key: &str,
    ) -> Result<Option<(String, Option<String>, Option<i64>, Option<bool>)>, AppError> {
        use super::entities::app_settings;

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
        use super::entities::app_settings;

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

    // 邀请码管理方法

    /// 创建邀请码
    pub async fn create_invite_code(
        &self,
        code: &str,
        created_by: i64,
        max_uses: i32,
        expires_at: Option<chrono::DateTime<Utc>>,
    ) -> Result<invite_codes::Model, AppError> {
        use super::entities::invite_codes;

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
        use super::entities::invite_codes;

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
        use super::entities::invite_codes;

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
        use super::entities::invite_codes;

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
        use super::entities::invite_codes;

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

    // 配置审计日志方法

    /// 记录配置变更
    pub async fn log_config_change(
        &self,
        config_key: &str,
        old_value: Option<String>,
        new_value: Option<String>,
        changed_by: i64,
        change_type: &str,
    ) -> Result<(), AppError> {
        use super::entities::config_audit_logs;

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
        use super::entities::config_audit_logs;

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
        use super::entities::config_audit_logs;

        let mut query = config_audit_logs::Entity::find()
            .filter(config_audit_logs::Column::ConfigKey.eq(config_key))
            .order_by_desc(config_audit_logs::Column::ChangedAt);

        if let Some(limit) = limit {
            query = query.limit(limit);
        }

        let logs = query.all(self.db.as_ref()).await?;
        Ok(logs)
    }

    // 用户授权管理方法

    /// 获取用户的授权列表
    pub async fn get_user_authorizations(
        &self,
        user_id: i64,
    ) -> Result<Vec<UserAuthorizationInfo>, AppError> {
        use super::entities::{access_tokens, o_auth_clients};

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
        use super::entities::{access_tokens, refresh_tokens};

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

/// 用户授权信息
#[derive(Debug, serde::Serialize)]
pub struct UserAuthorizationInfo {
    pub client_id: String,
    pub client_name: String,
    pub scopes: Vec<String>,
    pub granted_at: sea_orm::prelude::DateTimeWithTimeZone,
}

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
