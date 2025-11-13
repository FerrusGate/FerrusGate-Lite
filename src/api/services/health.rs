use actix_web::{web, HttpResponse};
use sea_orm::{DatabaseConnection, DbErr};
use std::sync::Arc;

use crate::cache::CompositeCache;

/// GET /health
pub async fn health_check() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

/// GET /health/ready
pub async fn readiness(
    db: web::Data<Arc<DatabaseConnection>>,
    cache: web::Data<Arc<CompositeCache>>,
) -> HttpResponse {
    // 检查数据库连接
    let db_status = match check_database_connection(&db).await {
        Ok(_) => "connected",
        Err(_) => "disconnected",
    };

    // 检查缓存系统
    let cache_status = check_cache_connection(&cache).await;

    let is_ready = db_status == "connected";

    let status_code = if is_ready {
        actix_web::http::StatusCode::OK
    } else {
        actix_web::http::StatusCode::SERVICE_UNAVAILABLE
    };

    HttpResponse::build(status_code).json(serde_json::json!({
        "status": if is_ready { "ready" } else { "not_ready" },
        "database": db_status,
        "cache": cache_status,
    }))
}

/// GET /health/live
pub async fn liveness() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "alive",
    }))
}

/// 检查数据库连接
async fn check_database_connection(db: &DatabaseConnection) -> Result<(), DbErr> {
    db.ping().await
}

/// 检查缓存连接
async fn check_cache_connection(cache: &CompositeCache) -> &'static str {
    // 简单测试缓存是否可用
    let test_key = "__health_check__";
    cache.set(test_key, "ok".to_string(), Some(5)).await;

    match cache.get(test_key).await {
        Some(_) => {
            cache.delete(test_key).await;
            "connected"
        }
        None => "degraded", // Redis 可能不可用，但内存缓存应该工作
    }
}
