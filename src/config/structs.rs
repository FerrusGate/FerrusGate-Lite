use serde::{Deserialize, Serialize};

/// 应用程序配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    #[serde(default)]
    pub server: ServerConfig,
    #[serde(default)]
    pub database: DatabaseConfig,
    #[serde(default)]
    pub redis: RedisConfig,
    #[serde(default)]
    pub auth: AuthConfig,
    #[serde(default)]
    pub cache: CacheConfig,
    #[serde(default)]
    pub log: LogConfig,
}

/// 服务器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_server_host")]
    pub host: String,
    #[serde(default = "default_server_port")]
    pub port: u16,
}

/// 数据库配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    #[serde(default = "default_database_url")]
    pub url: String,
    #[serde(default = "default_database_max_connections")]
    pub max_connections: u32,
    #[serde(default = "default_database_min_connections")]
    pub min_connections: u32,
}

/// Redis 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisConfig {
    #[serde(default = "default_redis_url")]
    pub url: String,
    #[serde(default = "default_redis_pool_size")]
    pub pool_size: u32,
}

/// 认证配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    #[serde(default = "default_jwt_secret")]
    pub jwt_secret: String,
    #[serde(default = "default_access_token_expire")]
    pub access_token_expire: i64,
    #[serde(default = "default_refresh_token_expire")]
    pub refresh_token_expire: i64,
    #[serde(default = "default_authorization_code_expire")]
    pub authorization_code_expire: i64,
}

/// 缓存配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    #[serde(default = "default_enable_memory_cache")]
    pub enable_memory_cache: bool,
    #[serde(default = "default_memory_cache_size")]
    pub memory_cache_size: u64,
    #[serde(default = "default_enable_redis_cache")]
    pub enable_redis_cache: bool,
    #[serde(default = "default_cache_default_ttl")]
    pub default_ttl: u64,
}

/// 日志配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogConfig {
    #[serde(default = "default_log_level")]
    pub level: String,
    #[serde(default = "default_log_format")]
    pub format: String,
    #[serde(default)]
    pub file: Option<String>,
    #[serde(default = "default_enable_rotation")]
    pub enable_rotation: bool,
    #[serde(default = "default_max_backups")]
    pub max_backups: u32,
}

// ============ Default Functions ============

fn default_server_host() -> String {
    "127.0.0.1".to_string()
}

fn default_server_port() -> u16 {
    8080
}

fn default_database_url() -> String {
    "sqlite://ferrusgate.db?mode=rwc".to_string()
}

fn default_database_max_connections() -> u32 {
    10
}

fn default_database_min_connections() -> u32 {
    2
}

fn default_redis_url() -> String {
    "redis://127.0.0.1:6379".to_string()
}

fn default_redis_pool_size() -> u32 {
    10
}

fn default_jwt_secret() -> String {
    "CHANGE-THIS-SECRET-IN-PRODUCTION-MIN-32-CHARS".to_string()
}

fn default_access_token_expire() -> i64 {
    3600 // 1 hour
}

fn default_refresh_token_expire() -> i64 {
    2592000 // 30 days
}

fn default_authorization_code_expire() -> i64 {
    300 // 5 minutes
}

fn default_enable_memory_cache() -> bool {
    true
}

fn default_memory_cache_size() -> u64 {
    10000
}

fn default_enable_redis_cache() -> bool {
    true
}

fn default_cache_default_ttl() -> u64 {
    300 // 5 minutes
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_log_format() -> String {
    "pretty".to_string()
}

fn default_enable_rotation() -> bool {
    true
}

fn default_max_backups() -> u32 {
    5
}

// ============ Default Trait Implementations ============

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: default_server_host(),
            port: default_server_port(),
        }
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: default_database_url(),
            max_connections: default_database_max_connections(),
            min_connections: default_database_min_connections(),
        }
    }
}

impl Default for RedisConfig {
    fn default() -> Self {
        Self {
            url: default_redis_url(),
            pool_size: default_redis_pool_size(),
        }
    }
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            jwt_secret: default_jwt_secret(),
            access_token_expire: default_access_token_expire(),
            refresh_token_expire: default_refresh_token_expire(),
            authorization_code_expire: default_authorization_code_expire(),
        }
    }
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enable_memory_cache: default_enable_memory_cache(),
            memory_cache_size: default_memory_cache_size(),
            enable_redis_cache: default_enable_redis_cache(),
            default_ttl: default_cache_default_ttl(),
        }
    }
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: default_log_level(),
            format: default_log_format(),
            file: None,
            enable_rotation: default_enable_rotation(),
            max_backups: default_max_backups(),
        }
    }
}
