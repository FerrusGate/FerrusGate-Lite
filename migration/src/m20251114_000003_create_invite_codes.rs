use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 创建 invite_codes 表
        manager
            .create_table(
                Table::create()
                    .table(InviteCodes::Table)
                    .if_not_exists()
                    .col(pk_auto(InviteCodes::Id))
                    .col(string_uniq(InviteCodes::Code))
                    .col(integer(InviteCodes::CreatedBy)) // 哪个管理员创建的
                    .col(integer_null(InviteCodes::UsedBy)) // 被哪个用户使用
                    .col(integer(InviteCodes::MaxUses).default(1)) // 最多使用次数
                    .col(integer(InviteCodes::UsedCount).default(0)) // 已使用次数
                    .col(timestamp_with_time_zone_null(InviteCodes::ExpiresAt)) // 过期时间（可选）
                    .col(timestamp_with_time_zone(InviteCodes::CreatedAt))
                    .foreign_key(
                        ForeignKey::create()
                            .from(InviteCodes::Table, InviteCodes::CreatedBy)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(InviteCodes::Table, InviteCodes::UsedBy)
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
                    .name("idx_invite_codes_code")
                    .table(InviteCodes::Table)
                    .col(InviteCodes::Code)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_invite_codes_expires_at")
                    .table(InviteCodes::Table)
                    .col(InviteCodes::ExpiresAt)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(InviteCodes::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum InviteCodes {
    Table,
    Id,
    Code,
    CreatedBy,
    UsedBy,
    MaxUses,
    UsedCount,
    ExpiresAt,
    CreatedAt,
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
}
