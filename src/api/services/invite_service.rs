use actix_web::{HttpMessage, HttpRequest, HttpResponse, web};
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::errors::AppError;
use crate::security::Claims;
use crate::storage::SeaOrmBackend;

#[derive(Debug, Deserialize)]
pub struct CreateInviteRequest {
    pub max_uses: Option<i32>,
    pub expires_in_hours: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct CreateInviteResponse {
    pub code: String,
    pub max_uses: i32,
    pub expires_at: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct InviteCodeInfo {
    pub code: String,
    pub created_by: i64,
    pub used_by: Option<i64>,
    pub used_count: i64,
    pub max_uses: i64,
    pub expires_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct ListInvitesResponse {
    pub invites: Vec<InviteCodeInfo>,
}

#[derive(Debug, Deserialize)]
pub struct VerifyInviteRequest {
    pub code: String,
}

#[derive(Debug, Serialize)]
pub struct VerifyInviteResponse {
    pub valid: bool,
    pub remaining_uses: Option<i32>,
    pub reason: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct MessageResponse {
    pub message: String,
}

/// 生成随机邀请码
fn generate_invite_code() -> String {
    use rand::Rng;
    let charset: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
    let mut rng = rand::rng();
    let code: String = (0..12)
        .map(|_| {
            let idx = rng.random_range(0..charset.len());
            charset[idx] as char
        })
        .collect();
    format!("INV-{}", code)
}

/// POST /api/admin/invites
/// 生成邀请码
pub async fn create_invite(
    req: HttpRequest,
    body: web::Json<CreateInviteRequest>,
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

    // 生成邀请码
    let code = generate_invite_code();
    let max_uses = body.max_uses.unwrap_or(1);
    let expires_at = body
        .expires_in_hours
        .map(|hours| Utc::now() + Duration::hours(hours));

    // 保存到数据库
    let invite = storage
        .create_invite_code(&code, user_id, max_uses, expires_at)
        .await?;

    tracing::info!("Invite code created: {} by user {}", code, user_id);

    Ok(HttpResponse::Created().json(CreateInviteResponse {
        code: invite.code,
        max_uses: invite.max_uses as i32,
        expires_at: invite.expires_at.map(|dt| dt.to_rfc3339()),
    }))
}

/// GET /api/admin/invites
/// 列出所有邀请码
pub async fn list_invites(
    storage: web::Data<Arc<SeaOrmBackend>>,
) -> Result<HttpResponse, AppError> {
    let invites = storage.list_invite_codes().await?;

    let invite_infos: Vec<InviteCodeInfo> = invites
        .into_iter()
        .map(|inv| InviteCodeInfo {
            code: inv.code,
            created_by: inv.created_by,
            used_by: inv.used_by,
            used_count: inv.used_count,
            max_uses: inv.max_uses,
            expires_at: inv.expires_at.map(|dt| dt.to_rfc3339()),
            created_at: inv.created_at.to_rfc3339(),
        })
        .collect();

    Ok(HttpResponse::Ok().json(ListInvitesResponse {
        invites: invite_infos,
    }))
}

/// DELETE /api/admin/invites/{code}
/// 撤销邀请码
pub async fn revoke_invite(
    code: web::Path<String>,
    storage: web::Data<Arc<SeaOrmBackend>>,
) -> Result<HttpResponse, AppError> {
    storage.revoke_invite_code(&code).await?;

    tracing::info!("Invite code revoked: {}", code.as_str());

    Ok(HttpResponse::Ok().json(MessageResponse {
        message: "Invite code revoked".to_string(),
    }))
}

/// POST /api/auth/verify-invite
/// 验证邀请码（公开接口）
pub async fn verify_invite(
    body: web::Json<VerifyInviteRequest>,
    storage: web::Data<Arc<SeaOrmBackend>>,
) -> Result<HttpResponse, AppError> {
    match storage.find_invite_code(&body.code).await? {
        Some(invite) => {
            // 检查是否过期
            if let Some(expires_at) = invite.expires_at
                && Utc::now().timestamp() > expires_at.timestamp()
            {
                return Ok(HttpResponse::Ok().json(VerifyInviteResponse {
                    valid: false,
                    remaining_uses: None,
                    reason: Some("expired".to_string()),
                }));
            }

            // 检查使用次数
            if invite.used_count >= invite.max_uses {
                return Ok(HttpResponse::Ok().json(VerifyInviteResponse {
                    valid: false,
                    remaining_uses: None,
                    reason: Some("used_up".to_string()),
                }));
            }

            // 有效
            let remaining = invite.max_uses - invite.used_count;
            Ok(HttpResponse::Ok().json(VerifyInviteResponse {
                valid: true,
                remaining_uses: Some(remaining as i32),
                reason: None,
            }))
        }
        None => Ok(HttpResponse::Ok().json(VerifyInviteResponse {
            valid: false,
            remaining_uses: None,
            reason: Some("not_found".to_string()),
        })),
    }
}

/// GET /api/admin/invites/stats
/// 获取邀请码统计信息
pub async fn get_stats(storage: web::Data<Arc<SeaOrmBackend>>) -> Result<HttpResponse, AppError> {
    let stats = storage.get_invite_stats().await?;
    Ok(HttpResponse::Ok().json(stats))
}
