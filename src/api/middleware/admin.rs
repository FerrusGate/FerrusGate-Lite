use actix_web::{
    Error, HttpMessage,
    dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready},
};
use futures_util::future::LocalBoxFuture;
use std::future::{Ready, ready};
use std::rc::Rc;
use std::sync::Arc;

use crate::cache::CompositeCache;
use crate::errors::AppError;
use crate::security::JwtManager;
use crate::storage::{SeaOrmBackend, repository::UserRepository};

/// 管理员权限中间件
/// 要求用户必须是管理员角色才能访问
pub struct AdminOnly {
    jwt_manager: Arc<JwtManager>,
    cache: Arc<CompositeCache>,
    storage: Arc<SeaOrmBackend>,
}

impl AdminOnly {
    pub fn new(
        jwt_manager: Arc<JwtManager>,
        cache: Arc<CompositeCache>,
        storage: Arc<SeaOrmBackend>,
    ) -> Self {
        Self {
            jwt_manager,
            cache,
            storage,
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for AdminOnly
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = AdminOnlyMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AdminOnlyMiddleware {
            service: Rc::new(service),
            jwt_manager: self.jwt_manager.clone(),
            cache: self.cache.clone(),
            storage: self.storage.clone(),
        }))
    }
}

pub struct AdminOnlyMiddleware<S> {
    service: Rc<S>,
    jwt_manager: Arc<JwtManager>,
    cache: Arc<CompositeCache>,
    storage: Arc<SeaOrmBackend>,
}

impl<S, B> Service<ServiceRequest> for AdminOnlyMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let jwt_manager = self.jwt_manager.clone();
        let cache = self.cache.clone();
        let storage = self.storage.clone();
        let service = self.service.clone();

        // 提取 Authorization header
        let token = match extract_bearer_token(&req) {
            Ok(t) => t,
            Err(e) => return Box::pin(async move { Err(e.into()) }),
        };

        Box::pin(async move {
            // 检查黑名单
            if cache.exists(&format!("blacklist:{}", token)).await {
                return Err(AppError::TokenExpired.into());
            }

            // 验证 JWT
            let claims = jwt_manager
                .verify_token(&token)
                .map_err(|e| -> Error { e.into() })?;

            // 从数据库查询用户以确认 role
            let user_id = claims
                .sub
                .parse::<i64>()
                .map_err(|_| AppError::Unauthorized)?;

            let user = storage
                .find_by_id(user_id)
                .await
                .map_err(|e| -> Error { e.into() })?
                .ok_or(AppError::Unauthorized)?;

            // 检查用户角色是否为 admin
            if user.role != "admin" {
                return Err(AppError::Forbidden("Admin access required".to_string()).into());
            }

            // 将 Claims 注入到请求扩展中
            req.extensions_mut().insert(claims);

            // 调用下一个服务
            service.call(req).await
        })
    }
}

/// 从请求中提取 Bearer Token
fn extract_bearer_token(req: &ServiceRequest) -> Result<String, AppError> {
    req.headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer ").map(|h_strip| h_strip.to_string()))
        .ok_or(AppError::Unauthorized)
}
