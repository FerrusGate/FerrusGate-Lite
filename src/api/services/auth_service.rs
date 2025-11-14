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
    pub invite_code: Option<String>, // 邀请码（可选）
}

#[derive(Debug, Serialize)]
pub struct RegisterResponse {
    pub user_id: i64,
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
    // 0. 读取注册配置
    let config = storage.get_registration_config().await?;

    // 1. 检查是否允许注册
    if !config.allow_registration {
        return Err(AppError::BadRequest("Registration is disabled".into()));
    }

    // 2. 验证邮箱后缀
    if !config.allowed_email_domains.is_empty() {
        let domain = req
            .email
            .split('@')
            .nth(1)
            .ok_or_else(|| AppError::BadRequest("Invalid email format".into()))?;
        if !config.allowed_email_domains.iter().any(|d| d == domain) {
            return Err(AppError::BadRequest("Email domain not allowed".into()));
        }
    }

    // 3. 验证用户名长度
    if req.username.len() < config.min_username_length as usize
        || req.username.len() > config.max_username_length as usize
    {
        return Err(AppError::BadRequest(format!(
            "Username must be between {} and {} characters",
            config.min_username_length, config.max_username_length
        )));
    }

    // 4. 验证密码强度
    if req.password.len() < config.min_password_length as usize {
        return Err(AppError::BadRequest(format!(
            "Password must be at least {} characters",
            config.min_password_length
        )));
    }
    if config.password_require_uppercase && !req.password.chars().any(|c| c.is_uppercase()) {
        return Err(AppError::BadRequest(
            "Password must contain at least one uppercase letter".into(),
        ));
    }
    if config.password_require_lowercase && !req.password.chars().any(|c| c.is_lowercase()) {
        return Err(AppError::BadRequest(
            "Password must contain at least one lowercase letter".into(),
        ));
    }
    if config.password_require_numbers && !req.password.chars().any(|c| c.is_numeric()) {
        return Err(AppError::BadRequest(
            "Password must contain at least one number".into(),
        ));
    }
    if config.password_require_special {
        let special_chars = "!@#$%^&*()_+-=[]{}|;:,.<>?";
        if !req.password.chars().any(|c| special_chars.contains(c)) {
            return Err(AppError::BadRequest(
                "Password must contain at least one special character".into(),
            ));
        }
    }

    // 5. 验证邀请码（如果启用）
    if config.require_invite_code {
        let invite_code = req
            .invite_code
            .as_ref()
            .ok_or_else(|| AppError::BadRequest("Invite code required".into()))?;

        // 验证邀请码有效性（但暂时不标记为已使用）
        let invite = storage
            .find_invite_code(invite_code)
            .await?
            .ok_or_else(|| AppError::BadRequest("Invalid invite code".into()))?;

        // 检查是否过期
        if let Some(expires_at) = invite.expires_at
            && chrono::Utc::now().timestamp() > expires_at.timestamp()
        {
            return Err(AppError::BadRequest("Invite code expired".into()));
        }

        // 检查使用次数
        if invite.used_count >= invite.max_uses {
            return Err(AppError::BadRequest(
                "Invite code has been fully used".into(),
            ));
        }
    }

    // 6. 验证用户名唯一性
    if storage.find_by_username(&req.username).await?.is_some() {
        return Err(AppError::BadRequest("Username already exists".into()));
    }

    // 7. 验证邮箱唯一性
    if storage.find_by_email(&req.email).await?.is_some() {
        return Err(AppError::BadRequest("Email already exists".into()));
    }

    // 8. 加密密码
    let password_hash = PasswordManager::hash_password(&req.password)?;

    // 9. 创建用户（role 默认为 "user"）
    let user = storage
        .create(&req.username, &req.email, &password_hash)
        .await?;

    // 10. 如果使用了邀请码，标记为已使用
    if config.require_invite_code
        && let Some(invite_code) = &req.invite_code
    {
        storage
            .verify_and_use_invite_code(invite_code, user.id)
            .await?;
    }

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

    // 2. 检查用户状态
    if user.deleted_at.is_some() {
        return Err(AppError::Forbidden("User account has been deleted".into()));
    }

    if !user.is_active {
        return Err(AppError::Forbidden("User account is disabled".into()));
    }

    // 3. 验证密码
    if !PasswordManager::verify_password(&req.password, &user.password_hash)? {
        return Err(AppError::InvalidCredentials);
    }

    // 4. 更新登录信息
    let _ = storage.update_login_info(user.id).await; // 忽略错误，不影响登录

    // 5. 生成 Token
    let access_token = jwt_manager.generate_token(
        user.id as i64,
        config.auth.access_token_expire,
        Some(vec!["read".to_string(), "write".to_string()]),
        &user.role,
    )?;

    let refresh_token = jwt_manager.generate_token(
        user.id as i64,
        config.auth.refresh_token_expire,
        Some(vec!["refresh".to_string()]),
        &user.role,
    )?;

    // 6. 缓存 Token -> UserID 映射
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
