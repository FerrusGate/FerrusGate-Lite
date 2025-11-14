pub mod auth_service;
pub mod health;
pub mod oauth_service;
pub mod oidc_service;
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
