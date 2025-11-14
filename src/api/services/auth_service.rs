use actix_web::{HttpResponse, web};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::cache::CompositeCache;
use crate::errors::AppError;
use crate::security::{JwtManager, PasswordManager};
use crate::storage::{SeaOrmBackend, UserRepository};

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct RegisterResponse {
    pub user_id: i32,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

/// POST /api/auth/register
pub async fn register(
    req: web::Json<RegisterRequest>,
    storage: web::Data<Arc<SeaOrmBackend>>,
) -> Result<HttpResponse, AppError> {
    // 1. 验证用户名和邮箱唯一性
    if storage.find_by_username(&req.username).await?.is_some() {
        return Err(AppError::BadRequest("Username already exists".into()));
    }

    if storage.find_by_email(&req.email).await?.is_some() {
        return Err(AppError::BadRequest("Email already exists".into()));
    }

    // 2. 加密密码
    let password_hash = PasswordManager::hash_password(&req.password)?;

    // 3. 创建用户
    let user = storage
        .create(&req.username, &req.email, &password_hash)
        .await?;

    tracing::info!("User registered: {} (id: {})", user.username, user.id);

    Ok(HttpResponse::Created().json(RegisterResponse {
        user_id: user.id,
        message: "User created successfully".to_string(),
    }))
}

/// POST /api/auth/login
pub async fn login(
    req: web::Json<LoginRequest>,
    storage: web::Data<Arc<SeaOrmBackend>>,
    jwt_manager: web::Data<Arc<JwtManager>>,
    cache: web::Data<Arc<CompositeCache>>,
    config: web::Data<crate::config::AppConfig>,
) -> Result<HttpResponse, AppError> {
    // 1. 查找用户
    let user = storage
        .find_by_username(&req.username)
        .await?
        .ok_or(AppError::InvalidCredentials)?;

    // 2. 验证密码
    if !PasswordManager::verify_password(&req.password, &user.password_hash)? {
        return Err(AppError::InvalidCredentials);
    }

    // 3. 生成 Token
    let access_token = jwt_manager.generate_token(
        user.id as i64,
        config.auth.access_token_expire,
        Some(vec!["read".to_string(), "write".to_string()]),
    )?;

    let refresh_token = jwt_manager.generate_token(
        user.id as i64,
        config.auth.refresh_token_expire,
        Some(vec!["refresh".to_string()]),
    )?;

    // 4. 缓存 Token -> UserID 映射
    cache
        .set(
            &format!("token:{}", access_token),
            user.id.to_string(),
            Some(config.auth.access_token_expire as u64),
        )
        .await;

    tracing::info!("User logged in: {} (id: {})", user.username, user.id);

    Ok(HttpResponse::Ok().json(LoginResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: config.auth.access_token_expire,
    }))
}
