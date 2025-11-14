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
use crate::security::{Claims, JwtManager};

/// JWT 认证中间件
pub struct JwtAuth {
    jwt_manager: Arc<JwtManager>,
    cache: Arc<CompositeCache>,
}

impl JwtAuth {
    pub fn new(jwt_manager: Arc<JwtManager>, cache: Arc<CompositeCache>) -> Self {
        Self { jwt_manager, cache }
    }
}

impl<S, B> Transform<S, ServiceRequest> for JwtAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = JwtAuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(JwtAuthMiddleware {
            service: Rc::new(service),
            jwt_manager: self.jwt_manager.clone(),
            cache: self.cache.clone(),
        }))
    }
}

pub struct JwtAuthMiddleware<S> {
    service: Rc<S>,
    jwt_manager: Arc<JwtManager>,
    cache: Arc<CompositeCache>,
}

impl<S, B> Service<ServiceRequest> for JwtAuthMiddleware<S>
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
        .and_then(|h| {
            if h.starts_with("Bearer ") {
                Some(h[7..].to_string())
            } else {
                None
            }
        })
        .ok_or(AppError::Unauthorized)
}

/// 从请求扩展中提取 Claims
pub fn extract_claims(req: &ServiceRequest) -> Result<Claims, AppError> {
    req.extensions()
        .get::<Claims>()
        .cloned()
        .ok_or(AppError::Unauthorized)
}
