use actix_web::{App, HttpServer, middleware, web};
use std::sync::Arc;

use crate::api::{middleware as app_middleware, services};
use crate::config::get_config;
use crate::runtime::startup::StartupContext;
use crate::storage::SeaOrmBackend;

pub async fn run_server(ctx: StartupContext) -> std::io::Result<()> {
    let config = get_config();
    let bind_addr = format!("{}:{}", config.server.host, config.server.port);

    tracing::info!("Starting HTTP server on {}", bind_addr);

    // 创建存储后端（带缓存）
    let storage = Arc::new(SeaOrmBackend::with_cache(ctx.db.clone(), ctx.cache.clone()));

    HttpServer::new(move || {
        App::new()
            // 共享状态
            .app_data(web::Data::new(ctx.db.clone()))
            .app_data(web::Data::new(storage.clone()))
            .app_data(web::Data::new(ctx.cache.clone()))
            .app_data(web::Data::new(ctx.jwt_manager.clone()))
            // 中间件
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .wrap(middleware::DefaultHeaders::new().add(("X-Version", env!("CARGO_PKG_VERSION"))))
            // 健康检查端点（无需认证）
            .service(
                web::scope("/health")
                    .route("", web::get().to(services::health_check))
                    .route("/ready", web::get().to(services::readiness))
                    .route("/live", web::get().to(services::liveness)),
            )
            // 认证 API（无需认证）
            .service(
                web::scope("/api/auth")
                    .route("/register", web::post().to(services::register))
                    .route("/login", web::post().to(services::login))
                    .route("/verify-invite", web::post().to(services::invite_verify)),
            )
            // OAuth2 授权端点
            .service(
                web::scope("/oauth")
                    .route("/authorize", web::get().to(services::oauth_authorize))
                    .route("/token", web::post().to(services::oauth_token))
                    .route(
                        "/userinfo",
                        web::get()
                            .to(services::oidc_userinfo)
                            .wrap(app_middleware::JwtAuth::new(
                                ctx.jwt_manager.clone(),
                                ctx.cache.clone(),
                            )),
                    ),
            )
            // OIDC Discovery 端点（无需认证）
            .service(
                web::scope("/.well-known")
                    .route(
                        "/openid-configuration",
                        web::get().to(services::oidc_discovery),
                    )
                    .route("/jwks.json", web::get().to(services::oidc_jwks)),
            )
            // 用户 API（需要 JWT 认证）
            .service(
                web::scope("/api/user")
                    .wrap(app_middleware::JwtAuth::new(
                        ctx.jwt_manager.clone(),
                        ctx.cache.clone(),
                    ))
                    .route("/me", web::get().to(services::user_get_profile))
                    .route(
                        "/authorizations",
                        web::get().to(services::user_list_authorizations),
                    )
                    .route(
                        "/authorizations/{client_id}",
                        web::delete().to(services::user_revoke_authorization),
                    ),
            )
            // 管理员 API（需要管理员权限）
            .service(
                web::scope("/api/admin")
                    .wrap(app_middleware::AdminOnly::new(
                        ctx.jwt_manager.clone(),
                        ctx.cache.clone(),
                        storage.clone(),
                    ))
                    // 配置管理
                    .route(
                        "/settings/registration",
                        web::get().to(services::settings_get_registration_config),
                    )
                    .route(
                        "/settings/registration",
                        web::put().to(services::settings_update_registration_config),
                    )
                    .route(
                        "/settings/audit-logs",
                        web::get().to(services::settings_get_audit_logs),
                    )
                    // 邀请码管理
                    .route("/invites", web::post().to(services::invite_create))
                    .route("/invites", web::get().to(services::invite_list))
                    .route("/invites/stats", web::get().to(services::invite_get_stats))
                    .route("/invites/{code}", web::delete().to(services::invite_revoke))
                    // 用户管理
                    .route("/users", web::get().to(services::admin_list_users))
                    .route(
                        "/users/stats",
                        web::get().to(services::admin_get_user_stats),
                    )
                    .route("/users/{id}", web::get().to(services::admin_get_user))
                    .route(
                        "/users/{id}/role",
                        web::patch().to(services::admin_update_role),
                    )
                    .route(
                        "/users/{id}/status",
                        web::patch().to(services::admin_update_status),
                    )
                    .route(
                        "/users/{id}/reset-password",
                        web::post().to(services::admin_reset_password),
                    )
                    .route("/users/{id}", web::delete().to(services::admin_delete_user)),
            )
    })
    .bind(&bind_addr)?
    .run()
    .await
}
