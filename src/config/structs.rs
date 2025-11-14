use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub auth: AuthConfig,
    pub cache: CacheConfig,
    pub log: LogConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisConfig {
    pub url: String,
    pub pool_size: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub access_token_expire: i64,       // 秒
    pub refresh_token_expire: i64,      // 秒
    pub authorization_code_expire: i64, // 秒
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    pub enable_memory_cache: bool,
    pub memory_cache_size: u64, // 最大条目数
    pub enable_redis_cache: bool,
    pub default_ttl: u64, // 默认 TTL（秒）
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogConfig {
    pub level: String,         // trace, debug, info, warn, error
    pub format: String,        // pretty 或 json
    pub file: Option<String>,  // 日志文件路径，None 或空字符串表示输出到控制台
    pub enable_rotation: bool, // 是否启用日志轮转
    pub max_backups: u32,      // 保留的最大备份文件数
}
