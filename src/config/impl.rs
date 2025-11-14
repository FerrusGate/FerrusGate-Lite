use std::env;
use std::fs;
use std::path::Path;
use std::sync::OnceLock;

use super::AppConfig;

static CONFIG: OnceLock<AppConfig> = OnceLock::new();
static CONFIG_PATH: OnceLock<String> = OnceLock::new();

impl AppConfig {
    /// 从文件加载配置,支持环境变量覆盖
    ///
    /// # 参数
    /// * `config_path` - 可选的配置文件路径
    ///   - `Some(path)`: 使用指定文件（不存在则创建）
    ///   - `None`: 使用默认 "config.toml"（不存在则警告）
    pub fn load(config_path: Option<&str>) -> Self {
        let mut config = Self::load_from_file(config_path);
        config.override_with_env();
        config
    }

    /// 从 TOML 文件加载配置
    ///
    /// # 行为
    /// - 如果提供 `config_path` 且文件不存在: 创建默认配置文件
    /// - 如果未提供路径且文件不存在: 警告并使用内存默认值
    fn load_from_file(config_path: Option<&str>) -> Self {
        let path = config_path.unwrap_or("config.toml");
        let is_custom_path = config_path.is_some();

        // 检查文件是否存在
        if !Path::new(path).exists() {
            if is_custom_path {
                // 用户指定了自定义路径: 创建文件
                eprintln!("[WARN] 配置文件不存在: {}", path);
                eprintln!("[WARN] 正在创建默认配置文件...");
                if let Err(e) = Self::ensure_config_file(path) {
                    eprintln!("[ERROR] 创建配置文件失败 {}: {}", path, e);
                    eprintln!("[WARN] 使用内存默认配置");
                    return Self::default();
                }
                eprintln!("[INFO] 配置文件已创建: {}", path);
            } else {
                // 默认路径: 仅警告
                eprintln!("[WARN] 未找到配置文件: {}", path);
                eprintln!("[WARN] 使用内存默认配置");
                eprintln!("[HINT] 使用 -c/--config 指定自定义配置文件");
                return Self::default();
            }
        }

        // 加载文件
        match fs::read_to_string(path) {
            Ok(content) => match toml::from_str::<AppConfig>(&content) {
                Ok(config) => {
                    eprintln!("[INFO] 配置已从文件加载: {}", path);
                    config
                }
                Err(e) => {
                    eprintln!("[ERROR] 解析配置文件失败 {}: {}", path, e);
                    eprintln!("[WARN] 使用内存默认配置");
                    Self::default()
                }
            },
            Err(e) => {
                eprintln!("[ERROR] 读取配置文件失败 {}: {}", path, e);
                eprintln!("[WARN] 使用内存默认配置");
                Self::default()
            }
        }
    }

    /// 确保配置文件存在,不存在则创建默认值
    fn ensure_config_file(path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let default_config = Self::default();
        let content = toml::to_string_pretty(&default_config)?;

        // 如果需要,创建父目录
        if let Some(parent) = Path::new(path).parent()
            && !parent.as_os_str().is_empty() && !parent.exists() {
                fs::create_dir_all(parent)?;
            }

        fs::write(path, content)?;
        Ok(())
    }

    /// 用环境变量覆盖配置
    fn override_with_env(&mut self) {
        // 服务器配置
        if let Ok(host) = env::var("SERVER_HOST") {
            self.server.host = host;
        }
        if let Ok(port) = env::var("SERVER_PORT") {
            if let Ok(port) = port.parse() {
                self.server.port = port;
            } else {
                eprintln!("[ERROR] 无效的 SERVER_PORT: {}", port);
            }
        }

        // 数据库配置
        if let Ok(database_url) = env::var("DATABASE_URL") {
            self.database.url = database_url;
        }
        if let Ok(max_conn) = env::var("DATABASE_MAX_CONNECTIONS") {
            if let Ok(n) = max_conn.parse() {
                self.database.max_connections = n;
            } else {
                eprintln!("[ERROR] 无效的 DATABASE_MAX_CONNECTIONS: {}", max_conn);
            }
        }
        if let Ok(min_conn) = env::var("DATABASE_MIN_CONNECTIONS") {
            if let Ok(n) = min_conn.parse() {
                self.database.min_connections = n;
            } else {
                eprintln!("[ERROR] 无效的 DATABASE_MIN_CONNECTIONS: {}", min_conn);
            }
        }

        // Redis 配置
        if let Ok(redis_url) = env::var("REDIS_URL") {
            self.redis.url = redis_url;
        }
        if let Ok(pool_size) = env::var("REDIS_POOL_SIZE") {
            if let Ok(n) = pool_size.parse() {
                self.redis.pool_size = n;
            } else {
                eprintln!("[ERROR] 无效的 REDIS_POOL_SIZE: {}", pool_size);
            }
        }

        // 认证配置
        if let Ok(jwt_secret) = env::var("JWT_SECRET") {
            self.auth.jwt_secret = jwt_secret;
        }
        if let Ok(expire) = env::var("ACCESS_TOKEN_EXPIRE") {
            if let Ok(n) = expire.parse() {
                self.auth.access_token_expire = n;
            } else {
                eprintln!("[ERROR] 无效的 ACCESS_TOKEN_EXPIRE: {}", expire);
            }
        }
        if let Ok(expire) = env::var("REFRESH_TOKEN_EXPIRE") {
            if let Ok(n) = expire.parse() {
                self.auth.refresh_token_expire = n;
            } else {
                eprintln!("[ERROR] 无效的 REFRESH_TOKEN_EXPIRE: {}", expire);
            }
        }
        if let Ok(expire) = env::var("AUTHORIZATION_CODE_EXPIRE") {
            if let Ok(n) = expire.parse() {
                self.auth.authorization_code_expire = n;
            } else {
                eprintln!("[ERROR] 无效的 AUTHORIZATION_CODE_EXPIRE: {}", expire);
            }
        }

        // 缓存配置
        if let Ok(enable) = env::var("ENABLE_MEMORY_CACHE") {
            self.cache.enable_memory_cache = enable == "true" || enable == "1";
        }
        if let Ok(size) = env::var("MEMORY_CACHE_SIZE") {
            if let Ok(n) = size.parse() {
                self.cache.memory_cache_size = n;
            } else {
                eprintln!("[ERROR] 无效的 MEMORY_CACHE_SIZE: {}", size);
            }
        }
        if let Ok(enable) = env::var("ENABLE_REDIS_CACHE") {
            self.cache.enable_redis_cache = enable == "true" || enable == "1";
        }
        if let Ok(ttl) = env::var("CACHE_DEFAULT_TTL") {
            if let Ok(n) = ttl.parse() {
                self.cache.default_ttl = n;
            } else {
                eprintln!("[ERROR] 无效的 CACHE_DEFAULT_TTL: {}", ttl);
            }
        }

        // 日志配置
        if let Ok(level) = env::var("RUST_LOG") {
            self.log.level = level;
        }
        if let Ok(format) = env::var("LOG_FORMAT") {
            self.log.format = format;
        }
        if let Ok(file) = env::var("LOG_FILE") {
            self.log.file = Some(file);
        }
        if let Ok(enable) = env::var("LOG_ENABLE_ROTATION") {
            self.log.enable_rotation = enable == "true" || enable == "1";
        }
        if let Ok(backups) = env::var("LOG_MAX_BACKUPS") {
            if let Ok(n) = backups.parse() {
                self.log.max_backups = n;
            } else {
                eprintln!("[ERROR] 无效的 LOG_MAX_BACKUPS: {}", backups);
            }
        }
    }

    /// 生成示例 TOML 配置文件
    pub fn generate_sample_config() -> String {
        let sample_config = AppConfig::default();
        toml::to_string_pretty(&sample_config)
            .unwrap_or_else(|e| format!("生成配置示例出错: {}", e))
    }

    /// 保存当前配置到 TOML 文件
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let content = toml::to_string_pretty(self)?;

        // 如果需要,创建父目录
        if let Some(parent) = path.as_ref().parent()
            && !parent.as_os_str().is_empty() && !parent.exists() {
                fs::create_dir_all(parent)?;
            }

        fs::write(path, content)?;
        Ok(())
    }

    /// 验证配置有效性
    pub fn validate(&self) -> Result<(), String> {
        if self.auth.jwt_secret.len() < 32 {
            return Err("JWT secret 必须至少 32 个字符".to_string());
        }

        if self.auth.access_token_expire <= 0 {
            return Err("Access token 过期时间必须为正数".to_string());
        }

        if self.auth.refresh_token_expire <= 0 {
            return Err("Refresh token 过期时间必须为正数".to_string());
        }

        if self.auth.authorization_code_expire <= 0 {
            return Err("Authorization code 过期时间必须为正数".to_string());
        }

        Ok(())
    }
}

// ============ 全局配置实例 ============

/// 获取全局配置实例
pub fn get_config() -> &'static AppConfig {
    CONFIG.get().expect("配置未初始化,请先调用 init_config()")
}

/// 初始化全局配置
///
/// # 参数
/// * `config_path` - 可选的配置文件路径
///   - `Some(path)`: 从指定文件加载（不存在则创建）
///   - `None`: 从默认 "config.toml" 加载（不存在则警告）
///
/// # 示例
/// ```
/// // 使用默认 config.toml
/// init_config(None);
///
/// // 使用自定义配置文件
/// init_config(Some("custom.toml".to_string()));
/// ```
pub fn init_config(config_path: Option<String>) {
    // 存储配置路径供后续使用
    if let Some(path) = &config_path {
        CONFIG_PATH.set(path.clone()).ok();
    }

    // 初始化配置
    CONFIG.get_or_init(|| AppConfig::load(config_path.as_deref()));
}

/// 获取使用的配置文件路径
pub fn get_config_path() -> Option<&'static str> {
    CONFIG_PATH.get().map(|s| s.as_str())
}
