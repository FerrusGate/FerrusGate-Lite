use sea_orm::DatabaseConnection;
use std::sync::Arc;
use tracing_appender::non_blocking::WorkerGuard;

use crate::cache::{CompositeCache, MemoryCache, RedisCache};
use crate::config::{CacheConfig, RedisConfig, get_config};
use crate::errors::AppError;
use crate::security::JwtManager;
use crate::storage::{connect, run_migrations};

/// 服务器启动上下文
pub struct StartupContext {
    pub db: Arc<DatabaseConnection>,
    pub cache: Arc<CompositeCache>,
    pub jwt_manager: Arc<JwtManager>,
    _log_guard: WorkerGuard,
}

/// 初始化服务器
pub async fn prepare_server() -> Result<StartupContext, AppError> {
    // 获取全局配置
    let config = get_config();

    // 1. 初始化 Rust-TLS
    rustls::crypto::ring::default_provider()
        .install_default()
        .map_err(|_| AppError::Internal("Failed to install crypto provider".into()))?;

    // 2. 初始化日志
    let log_guard = crate::system::logging::init_logging(&config.log);
    tracing::info!("FerrusGate-Lite v0.0.1 starting...");

    // 3. 验证配置
    config.validate().map_err(AppError::Config)?;

    // 4. 初始化数据库
    tracing::info!("Connecting to database: {}", config.database.url);
    let db = connect(&config.database).await?;
    tracing::info!("Database connected");

    // 5. 运行数据库迁移
    tracing::info!("Running database migrations...");
    run_migrations(&db).await?;

    // 6. 初始化缓存
    tracing::info!("Initializing cache system...");
    let cache = init_cache(&config.cache, &config.redis).await?;
    tracing::info!("Cache initialized");

    // 7. 初始化 JWT 管理器
    let jwt_manager = Arc::new(JwtManager::new(config.auth.jwt_secret.clone()));
    tracing::info!("JWT manager initialized");

    // 8. 检查并显示组件状态
    check_components_status();

    tracing::info!("Server initialization complete");

    Ok(StartupContext {
        db: Arc::new(db),
        cache: Arc::new(cache),
        jwt_manager,
        _log_guard: log_guard,
    })
}

/// 初始化缓存系统
async fn init_cache(
    cache_config: &CacheConfig,
    redis_config: &RedisConfig,
) -> Result<CompositeCache, AppError> {
    let memory_cache = Arc::new(MemoryCache::new(cache_config.memory_cache_size));

    let redis_cache = match RedisCache::new(redis_config).await {
        Ok(cache) => {
            tracing::info!("Redis cache connected");
            Arc::new(cache) as Arc<dyn crate::cache::Cache>
        }
        Err(e) => {
            tracing::warn!("Redis connection failed: {}", e);
            tracing::warn!("Falling back to memory-only cache");
            // 使用内存缓存作为fallback
            memory_cache.clone() as Arc<dyn crate::cache::Cache>
        }
    };

    Ok(CompositeCache::new(
        memory_cache as Arc<dyn crate::cache::Cache>,
        redis_cache,
    ))
}

/// 检查并显示组件状态
fn check_components_status() {
    let config = get_config();

    tracing::info!("==========================================");
    tracing::info!("  组件状态检查");
    tracing::info!("==========================================");

    // 检查服务器配置
    tracing::info!(
        "[OK] HTTP 服务器: {}:{}",
        config.server.host,
        config.server.port
    );

    // 检查数据库
    tracing::info!("[OK] 数据库: {}", config.database.url);

    // 检查缓存配置
    if config.cache.enable_memory_cache {
        tracing::info!(
            "[OK] 内存缓存: 已启用 (容量: {})",
            config.cache.memory_cache_size
        );
    } else {
        tracing::warn!("[-] 内存缓存: 已禁用");
    }

    if config.cache.enable_redis_cache {
        tracing::info!("[OK] Redis 缓存: 已启用 ({})", config.redis.url);
    } else {
        tracing::warn!("[-] Redis 缓存: 已禁用");
    }

    // 检查 JWT 配置
    if config.auth.jwt_secret.len() >= 32 {
        tracing::info!("[OK] JWT 认证: 已配置");
    } else {
        tracing::warn!("[!] JWT Secret 长度不足 32 字符");
    }

    // 检查 Token 过期时间
    tracing::info!(
        "[OK] Token 配置: Access={}s, Refresh={}s, AuthCode={}s",
        config.auth.access_token_expire,
        config.auth.refresh_token_expire,
        config.auth.authorization_code_expire
    );

    // 检查功能端点
    tracing::info!("[OK] OAuth2/OIDC: 已启用");
    tracing::info!("  - /oauth/authorize    (授权端点)");
    tracing::info!("  - /oauth/token        (Token 端点)");
    tracing::info!("  - /oauth/userinfo     (用户信息)");
    tracing::info!("  - /.well-known/openid-configuration (Discovery)");

    tracing::info!("==========================================");
}
