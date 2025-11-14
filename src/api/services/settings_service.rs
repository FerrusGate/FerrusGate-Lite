use actix_web::{HttpMessage, HttpRequest, HttpResponse, web};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::config::RegistrationConfig;
use crate::errors::AppError;
use crate::security::Claims;
use crate::storage::{SeaOrmBackend, entities::config_audit_logs};

#[derive(Debug, Serialize)]
pub struct SettingsUpdateResponse {
    pub message: String,
}

/// GET /api/admin/settings/registration
/// 获取注册配置
pub async fn get_registration_config(
    storage: web::Data<Arc<SeaOrmBackend>>,
) -> Result<HttpResponse, AppError> {
    let config = storage.get_registration_config().await?;
    Ok(HttpResponse::Ok().json(config))
}

/// PUT /api/admin/settings/registration
/// 更新注册配置
pub async fn update_registration_config(
    req: HttpRequest,
    config: web::Json<RegistrationConfig>,
    storage: web::Data<Arc<SeaOrmBackend>>,
) -> Result<HttpResponse, AppError> {
    // 从请求扩展中提取 Claims（由 AdminOnly 中间件注入）
    let claims = req
        .extensions()
        .get::<Claims>()
        .cloned()
        .ok_or(AppError::Unauthorized)?;

    // 解析 user_id
    let user_id: i64 = claims
        .sub
        .parse()
        .map_err(|_| AppError::Internal("Invalid user_id in token".into()))?;

    // 更新配置
    storage
        .update_registration_config(&config.into_inner(), user_id)
        .await?;

    Ok(HttpResponse::Ok().json(SettingsUpdateResponse {
        message: "Configuration updated successfully".to_string(),
    }))
}

#[derive(Debug, Deserialize)]
pub struct AuditLogsQuery {
    pub limit: Option<u64>,
    pub config_key: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AuditLogsResponse {
    pub logs: Vec<config_audit_logs::Model>,
}

/// GET /api/admin/settings/audit-logs
/// 获取配置变更审计日志
pub async fn get_audit_logs(
    query: web::Query<AuditLogsQuery>,
    storage: web::Data<Arc<SeaOrmBackend>>,
) -> Result<HttpResponse, AppError> {
    let logs = if let Some(config_key) = &query.config_key {
        storage
            .get_config_audit_logs_by_key(config_key, query.limit)
            .await?
    } else {
        storage.get_config_audit_logs(query.limit).await?
    };

    Ok(HttpResponse::Ok().json(AuditLogsResponse { logs }))
}
