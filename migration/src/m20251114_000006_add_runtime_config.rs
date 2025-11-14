use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 插入运行时配置项（Token 过期时间、缓存配置）
        let runtime_configs = vec![
            // ========== 认证策略配置 ==========
            // Access Token 过期时间（秒）
            (
                "access_token_expire",
                "int",
                3600i64,
                "Access Token 过期时间（秒）",
            ),
            // Refresh Token 过期时间（秒）
            (
                "refresh_token_expire",
                "int",
                2592000i64, // 30 天
                "Refresh Token 过期时间（秒）",
            ),
            // 授权码过期时间（秒）
            (
                "authorization_code_expire",
                "int",
                300i64, // 5 分钟
                "OAuth2 授权码过期时间（秒）",
            ),
            // ========== 缓存策略配置 ==========
            // 默认缓存 TTL（秒）
            (
                "default_ttl",
                "int",
                300i64, // 5 分钟
                "默认缓存过期时间（秒）",
            ),
        ];

        for (key, value_type, value_int, description) in runtime_configs {
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
                    value_int.into(),
                    description.into(),
                    SimpleExpr::Custom("CURRENT_TIMESTAMP".into()),
                ])
                .to_owned();
            manager.exec_stmt(insert).await?;
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 删除插入的运行时配置项
        let keys = vec![
            "access_token_expire",
            "refresh_token_expire",
            "authorization_code_expire",
            "default_ttl",
        ];

        for key in keys {
            let delete = Query::delete()
                .from_table(AppSettings::Table)
                .and_where(Expr::col(AppSettings::Key).eq(key))
                .to_owned();
            manager.exec_stmt(delete).await?;
        }

        Ok(())
    }
}

#[derive(DeriveIden)]
enum AppSettings {
    Table,
    Key,
    ValueType,
    ValueInt,
    Description,
    UpdatedAt,
}
