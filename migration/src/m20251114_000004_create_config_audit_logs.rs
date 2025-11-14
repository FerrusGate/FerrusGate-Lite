use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 创建 config_audit_logs 表
        manager
            .create_table(
                Table::create()
                    .table(ConfigAuditLogs::Table)
                    .if_not_exists()
                    .col(pk_auto(ConfigAuditLogs::Id))
                    .col(string(ConfigAuditLogs::ConfigKey)) // 配置键名
                    .col(text_null(ConfigAuditLogs::OldValue)) // 旧值（JSON格式）
                    .col(text_null(ConfigAuditLogs::NewValue)) // 新值（JSON格式）
                    .col(integer(ConfigAuditLogs::ChangedBy)) // 修改者用户ID
                    .col(timestamp_with_time_zone(ConfigAuditLogs::ChangedAt)) // 修改时间
                    .col(string_null(ConfigAuditLogs::ChangeType)) // 变更类型: create, update, delete
                    .foreign_key(
                        ForeignKey::create()
                            .from(ConfigAuditLogs::Table, ConfigAuditLogs::ChangedBy)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // 创建索引
        manager
            .create_index(
                Index::create()
                    .name("idx_config_audit_logs_key")
                    .table(ConfigAuditLogs::Table)
                    .col(ConfigAuditLogs::ConfigKey)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_config_audit_logs_changed_at")
                    .table(ConfigAuditLogs::Table)
                    .col(ConfigAuditLogs::ChangedAt)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ConfigAuditLogs::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum ConfigAuditLogs {
    Table,
    Id,
    ConfigKey,
    OldValue,
    NewValue,
    ChangedBy,
    ChangedAt,
    ChangeType,
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
}
