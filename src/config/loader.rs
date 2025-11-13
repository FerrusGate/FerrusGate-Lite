use std::fs;
use crate::config::structs::AppConfig;
use crate::errors::AppError;

impl AppConfig {
    /// 从文件加载配置
    pub fn from_file(path: &str) -> Result<Self, AppError> {
        let content = fs::read_to_string(path)
            .map_err(|e| AppError::Config(format!("Failed to read config file: {}", e)))?;

        let config: AppConfig = toml::from_str(&content)
            .map_err(|e| AppError::Config(format!("Failed to parse config: {}", e)))?;

        Ok(config)
    }

    /// 加载默认配置文件（config.toml）
    pub fn load() -> Result<Self, AppError> {
        Self::from_file("config.toml")
    }

    /// 验证配置有效性
    pub fn validate(&self) -> Result<(), AppError> {
        if self.auth.jwt_secret.len() < 32 {
            return Err(AppError::Config(
                "JWT secret must be at least 32 characters".into()
            ));
        }

        if self.auth.access_token_expire <= 0 {
            return Err(AppError::Config(
                "Access token expire time must be positive".into()
            ));
        }

        Ok(())
    }
}
