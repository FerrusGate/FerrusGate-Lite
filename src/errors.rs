use actix_web::{HttpResponse, ResponseError, http::StatusCode};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    // 数据库错误
    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),

    // Redis 错误
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),

    // 认证错误
    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Token expired")]
    TokenExpired,

    #[error("Invalid token")]
    InvalidToken,

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Forbidden: {0}")]
    Forbidden(String),

    // OAuth2 错误
    #[error("Invalid OAuth2 client")]
    InvalidClient,

    #[error("Invalid authorization code")]
    InvalidAuthCode,

    #[error("Invalid redirect URI")]
    InvalidRedirectUri,

    #[error("Invalid grant type")]
    InvalidGrantType,

    #[error("Invalid scope")]
    InvalidScope,

    // 通用错误
    #[error("Not found")]
    NotFound,

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Internal server error: {0}")]
    Internal(String),

    #[error("Configuration error: {0}")]
    Config(String),
}

impl AppError {
    /// 获取错误代码
    pub fn code(&self) -> &'static str {
        match self {
            AppError::Database(_) => "E001",
            AppError::Redis(_) => "E002",
            AppError::InvalidCredentials => "E003",
            AppError::TokenExpired => "E004",
            AppError::InvalidToken => "E005",
            AppError::Unauthorized => "E006",
            AppError::Forbidden(_) => "E016",
            AppError::InvalidClient => "E007",
            AppError::InvalidAuthCode => "E008",
            AppError::InvalidRedirectUri => "E009",
            AppError::InvalidGrantType => "E010",
            AppError::InvalidScope => "E011",
            AppError::NotFound => "E012",
            AppError::BadRequest(_) => "E013",
            AppError::Internal(_) => "E014",
            AppError::Config(_) => "E015",
        }
    }

    /// 获取错误类型名称
    pub fn error_type(&self) -> &'static str {
        match self {
            AppError::Database(_) => "Database Error",
            AppError::Redis(_) => "Redis Error",
            AppError::InvalidCredentials => "Invalid Credentials",
            AppError::TokenExpired => "Token Expired",
            AppError::InvalidToken => "Invalid Token",
            AppError::Unauthorized => "Unauthorized",
            AppError::Forbidden(_) => "Forbidden",
            AppError::InvalidClient => "Invalid Client",
            AppError::InvalidAuthCode => "Invalid Authorization Code",
            AppError::InvalidRedirectUri => "Invalid Redirect URI",
            AppError::InvalidGrantType => "Invalid Grant Type",
            AppError::InvalidScope => "Invalid Scope",
            AppError::NotFound => "Not Found",
            AppError::BadRequest(_) => "Bad Request",
            AppError::Internal(_) => "Internal Server Error",
            AppError::Config(_) => "Configuration Error",
        }
    }

    /// 获取错误详情
    pub fn message(&self) -> String {
        self.to_string()
    }

    /// 格式化为彩色输出（用于日志）
    pub fn format_colored(&self) -> String {
        use colored::Colorize;
        format!(
            "{} {} {}\n  {}",
            "[ERROR]".red().bold(),
            self.code().yellow(),
            self.error_type().red(),
            self.message().white()
        )
    }

    /// 格式化为简洁输出
    pub fn format_simple(&self) -> String {
        format!(
            "[{}] {}: {}",
            self.code(),
            self.error_type(),
            self.message()
        )
    }
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
    message: String,
}

impl ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            AppError::InvalidCredentials
            | AppError::TokenExpired
            | AppError::InvalidToken
            | AppError::Unauthorized => StatusCode::UNAUTHORIZED,

            AppError::Forbidden(_) => StatusCode::FORBIDDEN,

            AppError::NotFound => StatusCode::NOT_FOUND,

            AppError::BadRequest(_)
            | AppError::InvalidClient
            | AppError::InvalidAuthCode
            | AppError::InvalidRedirectUri
            | AppError::InvalidGrantType
            | AppError::InvalidScope => StatusCode::BAD_REQUEST,

            AppError::Database(_)
            | AppError::Redis(_)
            | AppError::Internal(_)
            | AppError::Config(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let status = self.status_code();
        let error_type = match self {
            AppError::InvalidCredentials => "invalid_credentials",
            AppError::TokenExpired => "token_expired",
            AppError::InvalidToken => "invalid_token",
            AppError::Unauthorized => "unauthorized",
            AppError::Forbidden(_) => "forbidden",
            AppError::InvalidClient => "invalid_client",
            AppError::InvalidAuthCode => "invalid_grant",
            AppError::InvalidRedirectUri => "invalid_request",
            AppError::InvalidGrantType => "unsupported_grant_type",
            AppError::InvalidScope => "invalid_scope",
            AppError::NotFound => "not_found",
            AppError::BadRequest(_) => "bad_request",
            _ => "internal_error",
        };

        HttpResponse::build(status).json(ErrorResponse {
            error: error_type.to_string(),
            message: self.to_string(),
        })
    }
}

// 为 Box<dyn std::error::Error> 实现转换
impl From<Box<dyn std::error::Error>> for AppError {
    fn from(err: Box<dyn std::error::Error>) -> Self {
        AppError::Internal(err.to_string())
    }
}
