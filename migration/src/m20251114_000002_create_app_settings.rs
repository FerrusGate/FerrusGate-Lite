use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 创建 app_settings 表 (类型化 key-value)
        manager
            .create_table(
                Table::create()
                    .table(AppSettings::Table)
                    .if_not_exists()
                    .col(pk_auto(AppSettings::Id))
                    .col(string_uniq(AppSettings::Key))
                    .col(string(AppSettings::ValueType)) // 'string', 'int', 'bool'
                    .col(text_null(AppSettings::ValueString))
                    .col(integer_null(AppSettings::ValueInt))
                    .col(boolean_null(AppSettings::ValueBool))
                    .col(text_null(AppSettings::Description)) // 配置项描述
                    .col(timestamp_with_time_zone(AppSettings::UpdatedAt))
                    .col(integer_null(AppSettings::UpdatedBy)) // 哪个管理员修改的
                    .foreign_key(
                        ForeignKey::create()
                            .from(AppSettings::Table, AppSettings::UpdatedBy)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        // 创建索引
        manager
            .create_index(
                Index::create()
                    .name("idx_app_settings_key")
                    .table(AppSettings::Table)
                    .col(AppSettings::Key)
                    .to_owned(),
            )
            .await?;

        // 插入默认注册配置
        let default_configs = vec![
            // 是否允许注册
            (
                "allow_registration",
                "bool",
                None,
                None,
                Some(true),
                "是否允许新用户注册",
            ),
            // 邮箱域名白名单（逗号分隔，空表示不限制）
            (
                "allowed_email_domains",
                "string",
                Some(""),
                None,
                None,
                "允许的邮箱后缀白名单，逗号分隔，留空表示不限制",
            ),
            // 用户名长度限制
            (
                "min_username_length",
                "int",
                None,
                Some(3),
                None,
                "用户名最小长度",
            ),
            (
                "max_username_length",
                "int",
                None,
                Some(32),
                None,
                "用户名最大长度",
            ),
            // 密码长度限制
            (
                "min_password_length",
                "int",
                None,
                Some(8),
                None,
                "密码最小长度",
            ),
            // 密码复杂度要求
            (
                "password_require_uppercase",
                "bool",
                None,
                None,
                Some(false),
                "密码是否需要大写字母",
            ),
            (
                "password_require_lowercase",
                "bool",
                None,
                None,
                Some(false),
                "密码是否需要小写字母",
            ),
            (
                "password_require_numbers",
                "bool",
                None,
                None,
                Some(false),
                "密码是否需要数字",
            ),
            (
                "password_require_special",
                "bool",
                None,
                None,
                Some(false),
                "密码是否需要特殊字符",
            ),
            // 邀请码机制
            (
                "require_invite_code",
                "bool",
                None,
                None,
                Some(false),
                "注册是否需要邀请码",
            ),
        ];

        for (key, value_type, value_string, value_int, value_bool, description) in default_configs {
            // 根据类型构造不同的插入语句
            if value_type == "string" {
                let insert = Query::insert()
                    .into_table(AppSettings::Table)
                    .columns([
                        AppSettings::Key,
                        AppSettings::ValueType,
                        AppSettings::ValueString,
                        AppSettings::Description,
                        AppSettings::UpdatedAt,
                    ])
                    .values_panic([
                        key.into(),
                        value_type.into(),
                        value_string.unwrap_or("").into(),
                        description.into(),
                        SimpleExpr::Custom("CURRENT_TIMESTAMP".into()),
                    ])
                    .to_owned();
                manager.exec_stmt(insert).await?;
            } else if value_type == "int" {
                let insert = Query::insert()
                    .into_table(AppSettings::Table)
                    .columns([
                        AppSettings::Key,
                        AppSettings::ValueType,
                        AppSettings::ValueInt,
                        AppSettings::Description,
                        AppSettings::UpdatedAt,
                    ])
                    .values_panic([
                        key.into(),
                        value_type.into(),
                        value_int.unwrap_or(0).into(),
                        description.into(),
                        SimpleExpr::Custom("CURRENT_TIMESTAMP".into()),
                    ])
                    .to_owned();
                manager.exec_stmt(insert).await?;
            } else if value_type == "bool" {
                let insert = Query::insert()
                    .into_table(AppSettings::Table)
                    .columns([
                        AppSettings::Key,
                        AppSettings::ValueType,
                        AppSettings::ValueBool,
                        AppSettings::Description,
                        AppSettings::UpdatedAt,
                    ])
                    .values_panic([
                        key.into(),
                        value_type.into(),
                        value_bool.unwrap_or(false).into(),
                        description.into(),
                        SimpleExpr::Custom("CURRENT_TIMESTAMP".into()),
                    ])
                    .to_owned();
                manager.exec_stmt(insert).await?;
            }
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AppSettings::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum AppSettings {
    Table,
    Id,
    Key,
    ValueType,
    ValueString,
    ValueInt,
    ValueBool,
    Description,
    UpdatedAt,
    UpdatedBy,
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
}
