use sea_orm::{Database, DatabaseConnection, ConnectOptions};
use migration::MigratorTrait;
use std::sync::Arc;
use std::time::Duration;

use crate::config::{AppConfig, DatabaseConfig, RedisConfig, CacheConfig};
use crate::errors::AppError;
use crate::cache::{MemoryCache, RedisCache, CompositeCache};
use crate::security::JwtManager;

/// 服务器启动上下文
pub struct StartupContext {
    pub db: Arc<DatabaseConnection>,
    pub cache: Arc<CompositeCache>,
    pub jwt_manager: Arc<JwtManager>,
    pub config: AppConfig,
}

/// 初始化服务器
pub async fn prepare_server(config: AppConfig) -> Result<StartupContext, AppError> {
    // 1. 初始化 Rust-TLS
    rustls::crypto::ring::default_provider()
        .install_default()
        .map_err(|_| AppError::Internal("Failed to install crypto provider".into()))?;

    // 2. 初始化日志
    crate::system::logging::init_logging(&config.log);
    tracing::info!("FerrusGate-Lite v0.0.1 starting...");

    // 3. 验证配置
    config.validate()?;

    // 4. 初始化数据库
    tracing::info!("Connecting to database: {}", config.database.url);
    let db = init_database(&config.database).await?;
    tracing::info!("✓ Database connected");

    // 5. 运行数据库迁移
    tracing::info!("Running database migrations...");
    migration::Migrator::up(&db, None).await
        .map_err(|e| AppError::Database(e))?;
    tracing::info!("✓ Migrations completed");

    // 6. 初始化缓存
    tracing::info!("Initializing cache system...");
    let cache = init_cache(&config.cache, &config.redis).await?;
    tracing::info!("✓ Cache initialized");

    // 7. 初始化 JWT 管理器
    let jwt_manager = Arc::new(JwtManager::new(config.auth.jwt_secret.clone()));
    tracing::info!("✓ JWT manager initialized");

    tracing::info!("Server initialization complete");

    Ok(StartupContext {
        db: Arc::new(db),
        cache: Arc::new(cache),
        jwt_manager,
        config,
    })
}

/// 初始化数据库连接
async fn init_database(config: &DatabaseConfig) -> Result<DatabaseConnection, AppError> {
    let mut opt = ConnectOptions::new(&config.url);
    opt.max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .connect_timeout(Duration::from_secs(8))
        .acquire_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .max_lifetime(Duration::from_secs(3600))
        .sqlx_logging(true);

    Database::connect(opt).await
        .map_err(|e| AppError::Database(e))
}

/// 初始化缓存系统
async fn init_cache(cache_config: &CacheConfig, redis_config: &RedisConfig) -> Result<CompositeCache, AppError> {
    let memory_cache = Arc::new(MemoryCache::new(cache_config.memory_cache_size));

    let redis_cache = match RedisCache::new(redis_config).await {
        Ok(cache) => {
            tracing::info!("✓ Redis cache connected");
            Arc::new(cache) as Arc<dyn crate::cache::Cache>
        }
        Err(e) => {
            tracing::warn!("⚠ Redis connection failed: {}", e);
            tracing::warn!("  Falling back to memory-only cache");
            // 使用内存缓存作为fallback
            memory_cache.clone() as Arc<dyn crate::cache::Cache>
        }
    };

    Ok(CompositeCache::new(
        memory_cache as Arc<dyn crate::cache::Cache>,
        redis_cache,
    ))
}
