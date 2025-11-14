use migration::{Migrator, MigratorTrait};
use sea_orm::sqlx::SqlitePool;
use sea_orm::sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqliteSynchronous};
use sea_orm::{ConnectOptions, Database, DatabaseConnection, SqlxSqliteConnector};
use std::str::FromStr;
use std::time::Duration;

use crate::config::DatabaseConfig;
use crate::errors::AppError;

/// 连接 SQLite 数据库（带自动创建和性能优化）
pub async fn connect_sqlite(database_url: &str) -> Result<DatabaseConnection, AppError> {
    let opt = SqliteConnectOptions::from_str(database_url)
        .map_err(|e| AppError::Internal(format!("SQLite URL 解析失败: {}", e)))?
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Normal)
        .busy_timeout(Duration::from_secs(5))
        .pragma("cache_size", "-64000") // 64MB cache
        .pragma("temp_store", "memory")
        .pragma("mmap_size", "536870912") // 512MB mmap
        .pragma("wal_autocheckpoint", "1000");

    let pool = SqlitePool::connect_with(opt)
        .await
        .map_err(|e| AppError::Internal(format!("无法连接到 SQLite 数据库: {}", e)))?;

    Ok(SqlxSqliteConnector::from_sqlx_sqlite_pool(pool))
}

/// 连接通用数据库（MySQL/PostgreSQL）
pub async fn connect_generic(
    database_url: &str,
    config: &DatabaseConfig,
) -> Result<DatabaseConnection, AppError> {
    let mut opt = ConnectOptions::new(database_url.to_owned());
    opt.max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .connect_timeout(Duration::from_secs(8))
        .acquire_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .max_lifetime(Duration::from_secs(3600));

    Database::connect(opt)
        .await
        .map_err(|e| AppError::Database(e))
}

/// 智能连接数据库（自动识别类型）
pub async fn connect(config: &DatabaseConfig) -> Result<DatabaseConnection, AppError> {
    if config.url.starts_with("sqlite://") || config.url.starts_with("sqlite:") {
        tracing::info!("使用 SQLite 数据库（已启用 WAL 和性能优化）");
        connect_sqlite(&config.url).await
    } else if config.url.starts_with("postgres://") || config.url.starts_with("postgresql://") {
        tracing::info!("使用 PostgreSQL 数据库");
        connect_generic(&config.url, config).await
    } else if config.url.starts_with("mysql://") {
        tracing::info!("使用 MySQL 数据库");
        connect_generic(&config.url, config).await
    } else {
        Err(AppError::Internal(format!(
            "不支持的数据库类型: {}",
            config.url
        )))
    }
}

/// 运行数据库迁移
pub async fn run_migrations(db: &DatabaseConnection) -> Result<(), AppError> {
    Migrator::up(db, None)
        .await
        .map_err(|e| AppError::Database(e))?;

    tracing::info!("✓ Database migrations completed");
    Ok(())
}
