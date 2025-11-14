use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // SQLite 不支持一次添加多个字段，需要分别执行

        // 添加 is_active 字段
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .add_column(boolean(Users::IsActive).default(true).not_null())
                    .to_owned(),
            )
            .await?;

        // 添加 deleted_at 字段
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .add_column(timestamp_with_time_zone_null(Users::DeletedAt))
                    .to_owned(),
            )
            .await?;

        // 添加 last_login_at 字段
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .add_column(timestamp_with_time_zone_null(Users::LastLoginAt))
                    .to_owned(),
            )
            .await?;

        // 添加 login_count 字段
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .add_column(big_integer(Users::LoginCount).default(0).not_null())
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // SQLite 不支持一次删除多个字段，需要分别执行

        // 删除 login_count 字段
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .drop_column(Users::LoginCount)
                    .to_owned(),
            )
            .await?;

        // 删除 last_login_at 字段
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .drop_column(Users::LastLoginAt)
                    .to_owned(),
            )
            .await?;

        // 删除 deleted_at 字段
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .drop_column(Users::DeletedAt)
                    .to_owned(),
            )
            .await?;

        // 删除 is_active 字段
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .drop_column(Users::IsActive)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Users {
    Table,
    IsActive,
    DeletedAt,
    LastLoginAt,
    LoginCount,
}
