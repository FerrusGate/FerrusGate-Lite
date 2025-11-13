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
