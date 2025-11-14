use async_trait::async_trait;
use chrono::Utc;
use sea_orm::*;

use crate::errors::AppError;
use crate::storage::entities::{prelude::*, *};
use crate::storage::repository::*;

use super::super::backend::SeaOrmBackend;

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

    async fn count_users(&self, _filter: UserListFilter) -> Result<UserStats, AppError> {
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
