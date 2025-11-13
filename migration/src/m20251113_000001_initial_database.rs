use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 创建 users 表
        manager
            .create_table(
                Table::create()
                    .table(Users::Table)
                    .if_not_exists()
                    .col(pk_auto(Users::Id))
                    .col(string_uniq(Users::Username))
                    .col(string_uniq(Users::Email))
                    .col(string(Users::PasswordHash))
                    .col(timestamp_with_time_zone(Users::CreatedAt))
                    .col(timestamp_with_time_zone(Users::UpdatedAt))
                    .to_owned(),
            )
            .await?;

        // 创建 oauth_clients 表
        manager
            .create_table(
                Table::create()
                    .table(OAuthClients::Table)
                    .if_not_exists()
                    .col(pk_auto(OAuthClients::Id))
                    .col(string_uniq(OAuthClients::ClientId))
                    .col(string(OAuthClients::ClientSecret))
                    .col(string(OAuthClients::Name))
                    .col(text(OAuthClients::RedirectUris)) // JSON array
                    .col(text(OAuthClients::AllowedScopes)) // JSON array
                    .col(timestamp_with_time_zone(OAuthClients::CreatedAt))
                    .to_owned(),
            )
            .await?;

        // 创建 authorization_codes 表
        manager
            .create_table(
                Table::create()
                    .table(AuthorizationCodes::Table)
                    .if_not_exists()
                    .col(pk_auto(AuthorizationCodes::Id))
                    .col(string_uniq(AuthorizationCodes::Code))
                    .col(string(AuthorizationCodes::ClientId))
                    .col(integer(AuthorizationCodes::UserId))
                    .col(text(AuthorizationCodes::RedirectUri))
                    .col(text(AuthorizationCodes::Scopes)) // JSON array
                    .col(timestamp_with_time_zone(AuthorizationCodes::ExpiresAt))
                    .col(boolean(AuthorizationCodes::Used).default(false))
                    .col(timestamp_with_time_zone(AuthorizationCodes::CreatedAt))
                    .foreign_key(
                        ForeignKey::create()
                            .from(AuthorizationCodes::Table, AuthorizationCodes::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // 创建 access_tokens 表
        manager
            .create_table(
                Table::create()
                    .table(AccessTokens::Table)
                    .if_not_exists()
                    .col(pk_auto(AccessTokens::Id))
                    .col(text(AccessTokens::Token))
                    .col(string(AccessTokens::TokenType).default("Bearer"))
                    .col(string(AccessTokens::ClientId))
                    .col(integer(AccessTokens::UserId))
                    .col(text(AccessTokens::Scopes)) // JSON array
                    .col(timestamp_with_time_zone(AccessTokens::ExpiresAt))
                    .col(timestamp_with_time_zone(AccessTokens::CreatedAt))
                    .foreign_key(
                        ForeignKey::create()
                            .from(AccessTokens::Table, AccessTokens::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // 创建 refresh_tokens 表
        manager
            .create_table(
                Table::create()
                    .table(RefreshTokens::Table)
                    .if_not_exists()
                    .col(pk_auto(RefreshTokens::Id))
                    .col(text(RefreshTokens::Token))
                    .col(integer(RefreshTokens::AccessTokenId))
                    .col(timestamp_with_time_zone(RefreshTokens::ExpiresAt))
                    .col(timestamp_with_time_zone(RefreshTokens::CreatedAt))
                    .foreign_key(
                        ForeignKey::create()
                            .from(RefreshTokens::Table, RefreshTokens::AccessTokenId)
                            .to(AccessTokens::Table, AccessTokens::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // 创建索引
        manager
            .create_index(
                Index::create()
                    .name("idx_users_username")
                    .table(Users::Table)
                    .col(Users::Username)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_users_email")
                    .table(Users::Table)
                    .col(Users::Email)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_oauth_clients_client_id")
                    .table(OAuthClients::Table)
                    .col(OAuthClients::ClientId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_auth_codes_code")
                    .table(AuthorizationCodes::Table)
                    .col(AuthorizationCodes::Code)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_auth_codes_expires_at")
                    .table(AuthorizationCodes::Table)
                    .col(AuthorizationCodes::ExpiresAt)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_access_tokens_expires_at")
                    .table(AccessTokens::Table)
                    .col(AccessTokens::ExpiresAt)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(RefreshTokens::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(AccessTokens::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(AuthorizationCodes::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(OAuthClients::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Users::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
    Username,
    Email,
    PasswordHash,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum OAuthClients {
    Table,
    Id,
    ClientId,
    ClientSecret,
    Name,
    RedirectUris,
    AllowedScopes,
    CreatedAt,
}

#[derive(DeriveIden)]
enum AuthorizationCodes {
    Table,
    Id,
    Code,
    ClientId,
    UserId,
    RedirectUri,
    Scopes,
    ExpiresAt,
    Used,
    CreatedAt,
}

#[derive(DeriveIden)]
enum AccessTokens {
    Table,
    Id,
    Token,
    TokenType,
    ClientId,
    UserId,
    Scopes,
    ExpiresAt,
    CreatedAt,
}

#[derive(DeriveIden)]
enum RefreshTokens {
    Table,
    Id,
    Token,
    AccessTokenId,
    ExpiresAt,
    CreatedAt,
}
