pub use sea_orm_migration::prelude::*;

mod m20251113_000001_initial_database;
mod m20251114_000001_add_user_role;
mod m20251114_000002_create_app_settings;
mod m20251114_000003_create_invite_codes;
mod m20251114_000004_create_config_audit_logs;
mod m20251114_000005_add_user_management_fields;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20251113_000001_initial_database::Migration),
            Box::new(m20251114_000001_add_user_role::Migration),
            Box::new(m20251114_000002_create_app_settings::Migration),
            Box::new(m20251114_000003_create_invite_codes::Migration),
            Box::new(m20251114_000004_create_config_audit_logs::Migration),
            Box::new(m20251114_000005_add_user_management_fields::Migration),
        ]
    }
}
