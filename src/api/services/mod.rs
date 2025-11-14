pub mod auth_service;
pub mod health;
pub mod invite_service;
pub mod oauth_service;
pub mod oidc_service;
pub mod settings_service;
pub mod user_service;

// 认证服务
pub use auth_service::{login, register};

// 健康检查
pub use health::{health_check, liveness, readiness};

// OAuth2 服务
pub use oauth_service::{authorize as oauth_authorize, token as oauth_token};

// OIDC 服务
pub use oidc_service::{discovery as oidc_discovery, jwks as oidc_jwks, userinfo as oidc_userinfo};

// 用户管理服务
pub use user_service::{
    get_profile as user_get_profile, list_authorizations as user_list_authorizations,
    revoke_authorization as user_revoke_authorization,
};

// 设置管理服务
pub use settings_service::{
    get_registration_config as settings_get_registration_config,
    update_registration_config as settings_update_registration_config,
};

// 邀请码服务
pub use invite_service::{
    create_invite as invite_create, get_stats as invite_get_stats, list_invites as invite_list,
    revoke_invite as invite_revoke, verify_invite as invite_verify,
};
