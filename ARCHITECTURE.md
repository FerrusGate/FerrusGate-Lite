# 📐 FerrusGate-Lite 详细架构设计文档

## 一、项目概述

**FerrusGate-Lite** 是 FerrusGate 的轻量级单租户版本，提供 OAuth2/OIDC 身份认证网关功能。

**核心特性：**
- 用户注册和登录认证
- OAuth2 授权码模式
- OIDC (OpenID Connect) 支持
- JWT Token 管理
- Redis 多层缓存
- 生产级可观测性

---

## 二、技术栈

| 层级 | 技术选型 | 说明 |
|------|---------|------|
| **Web 框架** | actix-web 4.11 | 高性能异步 HTTP 服务器 |
| **数据库 ORM** | SeaORM 2.0-rc | 异步 ORM，支持 SQLite/PostgreSQL/MySQL |
| **缓存** | Moka 0.12 + Redis 0.32 | L1 内存缓存 + L2 分布式缓存 |
| **异步运行时** | Tokio 1.45 | 多线程异步运行时 |
| **日志系统** | Tracing 0.1 | 结构化日志 |
| **序列化** | Serde 1.0 | JSON 序列化 |
| **密码加密** | bcrypt | Bcrypt 密码哈希 |
| **JWT** | jsonwebtoken | JWT 生成和验证 |
| **配置** | TOML | 配置文件格式 |

---

## 三、架构层级设计

```
┌─────────────────────────────────────────────────────┐
│                   HTTP Requests                     │
└─────────────────────┬───────────────────────────────┘
                      │
┌─────────────────────▼───────────────────────────────┐
│              Actix-Web Server                       │
│  ┌──────────────────────────────────────────────┐  │
│  │         Middleware Layer (中间件层)          │  │
│  │  - JWT Authentication                        │  │
│  │  - Rate Limiting (Redis)                     │  │
│  │  - CORS                                      │  │
│  │  - Request Logging                           │  │
│  └──────────────────┬───────────────────────────┘  │
└─────────────────────┼───────────────────────────────┘
                      │
┌─────────────────────▼───────────────────────────────┐
│               API Services (业务层)                  │
│  ┌──────────────┐  ┌──────────────┐  ┌───────────┐ │
│  │ AuthService  │  │ OAuthService │  │UserService│ │
│  │ - register   │  │ - authorize  │  │ - profile │ │
│  │ - login      │  │ - token      │  │ - update  │ │
│  └──────────────┘  └──────────────┘  └───────────┘ │
│  ┌──────────────┐  ┌──────────────┐                │
│  │ OIDCService  │  │HealthService │                │
│  │ - userinfo   │  │ - ready/live │                │
│  └──────────────┘  └──────────────┘                │
└─────────────────────┬───────────────────────────────┘
                      │
┌─────────────────────▼───────────────────────────────┐
│            Cache Layer (缓存层)                      │
│  ┌───────────────────────────────────────────────┐  │
│  │         Composite Cache (组合缓存)            │  │
│  │  ┌────────────────┐    ┌──────────────────┐  │  │
│  │  │ L1: Moka       │ ─▶ │ L2: Redis        │  │  │
│  │  │ (内存缓存)      │    │ (分布式缓存)      │  │  │
│  │  │ - Token        │    │ - Session        │  │  │
│  │  │ - UserInfo     │    │ - Token Blacklist│  │  │
│  │  └────────────────┘    └──────────────────┘  │  │
│  └───────────────────────────────────────────────┘  │
└─────────────────────┬───────────────────────────────┘
                      │
┌─────────────────────▼───────────────────────────────┐
│          Storage Layer (存储层)                      │
│  ┌───────────────────────────────────────────────┐  │
│  │      Repository Pattern (仓储模式)            │  │
│  │  - UserRepository                             │  │
│  │  - ClientRepository                           │  │
│  │  - TokenRepository                            │  │
│  └───────────────────┬───────────────────────────┘  │
└─────────────────────┼───────────────────────────────┘
                      │
┌─────────────────────▼───────────────────────────────┐
│              Database (SeaORM)                      │
│  ┌──────────┐  ┌───────────────┐  ┌─────────────┐  │
│  │  users   │  │ oauth_clients │  │access_tokens│  │
│  └──────────┘  └───────────────┘  └─────────────┘  │
│  ┌──────────────────┐  ┌────────────────────────┐  │
│  │authorization_codes│  │   refresh_tokens       │  │
│  └──────────────────┘  └────────────────────────┘  │
└─────────────────────────────────────────────────────┘
```

---

## 四、目录结构详解

```
FerrusGate-Lite/
├── Cargo.toml                      # 项目依赖配置
├── config.toml                     # 运行时配置文件
│
├── src/
│   ├── main.rs                     # 应用入口点
│   ├── lib.rs                      # 库根模块
│   ├── errors.rs                   # 统一错误类型定义
│   │
│   ├── api/                        # HTTP API 层
│   │   ├── mod.rs
│   │   ├── middleware/             # 中间件
│   │   │   ├── mod.rs
│   │   │   ├── auth.rs            # JWT 认证中间件
│   │   │   ├── rate_limit.rs     # Redis 限流中间件
│   │   │   └── cors.rs            # CORS 跨域处理
│   │   └── services/               # 业务服务
│   │       ├── mod.rs
│   │       ├── auth_service.rs    # 用户注册/登录
│   │       ├── oauth_service.rs   # OAuth2 授权流程
│   │       ├── oidc_service.rs    # OIDC UserInfo/Discovery
│   │       ├── user_service.rs    # 用户信息管理
│   │       └── health.rs          # 健康检查
│   │
│   ├── cache/                      # 缓存系统
│   │   ├── mod.rs
│   │   ├── traits.rs              # 缓存特征定义
│   │   ├── memory_cache.rs        # Moka 内存缓存
│   │   ├── redis_cache.rs         # Redis 缓存实现
│   │   └── composite.rs           # L1+L2 组合缓存
│   │
│   ├── storage/                    # 存储层
│   │   ├── mod.rs
│   │   ├── traits.rs              # Repository 特征
│   │   ├── backend.rs             # SeaORM 实现
│   │   └── models.rs              # SeaORM entities (生成)
│   │
│   ├── security/                   # 安全工具
│   │   ├── mod.rs
│   │   ├── jwt.rs                 # JWT 生成/验证
│   │   ├── password.rs            # Bcrypt 密码加密
│   │   └── token.rs               # OAuth2 Token 生成
│   │
│   ├── config/                     # 配置系统
│   │   ├── mod.rs
│   │   ├── structs.rs             # 配置结构体
│   │   └── loader.rs              # TOML 加载器
│   │
│   ├── runtime/                    # 运行时管理
│   │   ├── mod.rs
│   │   ├── startup.rs             # 服务器启动流程
│   │   ├── shutdown.rs            # 优雅关闭
│   │   └── server.rs              # Actix-Web 服务器配置
│   │
│   ├── system/                     # 系统层
│   │   ├── mod.rs
│   │   ├── logging.rs             # Tracing 日志初始化
│   │   └── signals.rs             # Unix/Windows 信号处理
│   │
│   └── utils/                      # 工具函数
│       └── mod.rs
│
└── migration/                      # 数据库迁移
    ├── Cargo.toml
    └── src/
        ├── lib.rs
        ├── main.rs
        └── m20251113_000001_initial.rs  # 初始迁移脚本
```

---

## 五、核心模块设计

### 5.1 错误处理 (errors.rs)

```rust
use actix_web::{HttpResponse, ResponseError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    // 数据库错误
    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),

    // Redis 错误
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),

    // 认证错误
    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Token expired")]
    TokenExpired,

    #[error("Invalid token")]
    InvalidToken,

    // OAuth2 错误
    #[error("Invalid OAuth2 client")]
    InvalidClient,

    #[error("Invalid authorization code")]
    InvalidAuthCode,

    #[error("Invalid redirect URI")]
    InvalidRedirectUri,

    // 通用错误
    #[error("Not found")]
    NotFound,

    #[error("Internal server error")]
    Internal(String),
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        match self {
            AppError::InvalidCredentials => HttpResponse::Unauthorized().json(...),
            AppError::TokenExpired => HttpResponse::Unauthorized().json(...),
            AppError::NotFound => HttpResponse::NotFound().json(...),
            _ => HttpResponse::InternalServerError().json(...),
        }
    }
}
```

---

### 5.2 配置系统 (config/)

**config/structs.rs:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub auth: AuthConfig,
    pub cache: CacheConfig,
    pub log: LogConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub access_token_expire: i64,    // 秒
    pub refresh_token_expire: i64,   // 秒
    pub authorization_code_expire: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    pub enable_memory_cache: bool,
    pub memory_cache_size: u64,      // 最大条目数
    pub enable_redis_cache: bool,
    pub default_ttl: u64,            // 默认 TTL（秒）
}
```

**config.toml 示例:**
```toml
[server]
host = "127.0.0.1"
port = 8080

[database]
url = "sqlite://ferrusgate.db?mode=rwc"
max_connections = 10
min_connections = 2

[redis]
url = "redis://127.0.0.1:6379"
pool_size = 10

[auth]
jwt_secret = "your-secret-key-change-in-production"
access_token_expire = 3600           # 1小时
refresh_token_expire = 2592000       # 30天
authorization_code_expire = 300      # 5分钟

[cache]
enable_memory_cache = true
memory_cache_size = 10000
enable_redis_cache = true
default_ttl = 300

[log]
level = "info"
format = "pretty"  # pretty 或 json
```

---

### 5.3 存储层 (storage/)

**storage/traits.rs:**
```rust
use async_trait::async_trait;
use crate::storage::models::*;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn create(&self, user: NewUser) -> Result<User, AppError>;
    async fn find_by_id(&self, id: i64) -> Result<Option<User>, AppError>;
    async fn find_by_username(&self, username: &str) -> Result<Option<User>, AppError>;
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, AppError>;
    async fn update(&self, user: User) -> Result<User, AppError>;
}

#[async_trait]
pub trait ClientRepository: Send + Sync {
    async fn find_by_client_id(&self, client_id: &str) -> Result<Option<OAuthClient>, AppError>;
    async fn create(&self, client: NewOAuthClient) -> Result<OAuthClient, AppError>;
    async fn verify_redirect_uri(&self, client_id: &str, redirect_uri: &str) -> Result<bool, AppError>;
}

#[async_trait]
pub trait TokenRepository: Send + Sync {
    async fn save_auth_code(&self, code: AuthorizationCode) -> Result<(), AppError>;
    async fn consume_auth_code(&self, code: &str) -> Result<Option<AuthCodeData>, AppError>;
    async fn save_access_token(&self, token: AccessToken) -> Result<(), AppError>;
    async fn find_token(&self, token: &str) -> Result<Option<TokenData>, AppError>;
    async fn revoke_token(&self, token: &str) -> Result<(), AppError>;
}
```

**storage/backend.rs:**
```rust
pub struct SeaOrmStorage {
    db: DatabaseConnection,
}

impl SeaOrmStorage {
    pub async fn new(config: &DatabaseConfig) -> Result<Self, AppError> {
        let db = Database::connect(&config.url).await?;
        Ok(Self { db })
    }
}

#[async_trait]
impl UserRepository for SeaOrmStorage {
    async fn find_by_username(&self, username: &str) -> Result<Option<User>, AppError> {
        use crate::storage::models::prelude::*;
        let user = Users::find()
            .filter(users::Column::Username.eq(username))
            .one(&self.db)
            .await?;
        Ok(user)
    }
    // ... 其他实现
}
```

---

### 5.4 缓存层 (cache/)

**cache/traits.rs:**
```rust
use async_trait::async_trait;

#[async_trait]
pub trait Cache: Send + Sync {
    async fn get(&self, key: &str) -> Option<String>;
    async fn set(&self, key: &str, value: String, ttl: Option<u64>);
    async fn delete(&self, key: &str);
    async fn exists(&self, key: &str) -> bool;
}
```

**cache/composite.rs:**
```rust
pub struct CompositeCache {
    l1: Arc<dyn Cache>,  // Moka 内存缓存
    l2: Arc<dyn Cache>,  // Redis 缓存
}

impl CompositeCache {
    pub fn new(l1: Arc<dyn Cache>, l2: Arc<dyn Cache>) -> Self {
        Self { l1, l2 }
    }

    pub async fn get(&self, key: &str) -> Option<String> {
        // 先查 L1
        if let Some(value) = self.l1.get(key).await {
            return Some(value);
        }
        // 再查 L2
        if let Some(value) = self.l2.get(key).await {
            // 回填 L1
            self.l1.set(key, value.clone(), None).await;
            return Some(value);
        }
        None
    }

    pub async fn set(&self, key: &str, value: String, ttl: Option<u64>) {
        self.l1.set(key, value.clone(), ttl).await;
        self.l2.set(key, value, ttl).await;
    }
}
```

**缓存使用场景：**
- **Token → UserID 映射**：加速 Token 验证（TTL: 1小时）
- **UserInfo 缓存**：减少用户信息查询（TTL: 5分钟）
- **Client 配置缓存**：OAuth2 客户端信息（TTL: 30分钟）
- **Token 黑名单**：撤销的 Token（仅 Redis，TTL: Token 过期时间）

---

### 5.5 安全层 (security/)

**security/jwt.rs:**
```rust
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,      // user_id
    pub exp: i64,         // 过期时间
    pub iat: i64,         // 签发时间
    pub scope: Vec<String>,  // 权限范围
}

pub struct JwtManager {
    secret: String,
}

impl JwtManager {
    pub fn new(secret: String) -> Self {
        Self { secret }
    }

    pub fn generate_token(&self, user_id: i64, expire_in: i64) -> Result<String, AppError> {
        let claims = Claims {
            sub: user_id.to_string(),
            exp: chrono::Utc::now().timestamp() + expire_in,
            iat: chrono::Utc::now().timestamp(),
            scope: vec!["read".to_string(), "write".to_string()],
        };

        encode(&Header::default(), &claims, &EncodingKey::from_secret(self.secret.as_ref()))
            .map_err(|_| AppError::Internal("JWT encode failed".into()))
    }

    pub fn verify_token(&self, token: &str) -> Result<Claims, AppError> {
        decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.secret.as_ref()),
            &Validation::default()
        )
        .map(|data| data.claims)
        .map_err(|_| AppError::InvalidToken)
    }
}
```

**security/password.rs:**
```rust
use bcrypt::{hash, verify, DEFAULT_COST};

pub struct PasswordManager;

impl PasswordManager {
    pub fn hash_password(password: &str) -> Result<String, AppError> {
        hash(password, DEFAULT_COST)
            .map_err(|_| AppError::Internal("Password hash failed".into()))
    }

    pub fn verify_password(password: &str, hash: &str) -> Result<bool, AppError> {
        verify(password, hash)
            .map_err(|_| AppError::Internal("Password verify failed".into()))
    }
}
```

---

### 5.6 API 服务层 (api/services/)

**api/services/auth_service.rs:**
```rust
pub struct AuthService {
    storage: Arc<dyn UserRepository>,
    jwt_manager: Arc<JwtManager>,
    cache: Arc<CompositeCache>,
}

impl AuthService {
    // POST /api/auth/register
    pub async fn register(&self, req: RegisterRequest) -> Result<RegisterResponse, AppError> {
        // 1. 验证用户名/邮箱唯一性
        if self.storage.find_by_username(&req.username).await?.is_some() {
            return Err(AppError::Internal("Username already exists".into()));
        }

        // 2. 密码加密
        let password_hash = PasswordManager::hash_password(&req.password)?;

        // 3. 创建用户
        let user = self.storage.create(NewUser {
            username: req.username,
            email: req.email,
            password_hash,
        }).await?;

        Ok(RegisterResponse { user_id: user.id })
    }

    // POST /api/auth/login
    pub async fn login(&self, req: LoginRequest) -> Result<LoginResponse, AppError> {
        // 1. 查找用户
        let user = self.storage.find_by_username(&req.username).await?
            .ok_or(AppError::InvalidCredentials)?;

        // 2. 验证密码
        if !PasswordManager::verify_password(&req.password, &user.password_hash)? {
            return Err(AppError::InvalidCredentials);
        }

        // 3. 生成 JWT Token
        let access_token = self.jwt_manager.generate_token(user.id, 3600)?;
        let refresh_token = self.jwt_manager.generate_token(user.id, 2592000)?;

        // 4. 缓存 Token
        self.cache.set(
            &format!("token:{}", access_token),
            user.id.to_string(),
            Some(3600)
        ).await;

        Ok(LoginResponse { access_token, refresh_token })
    }
}
```

**api/services/oauth_service.rs:**
```rust
pub struct OAuthService {
    client_repo: Arc<dyn ClientRepository>,
    token_repo: Arc<dyn TokenRepository>,
    jwt_manager: Arc<JwtManager>,
    cache: Arc<CompositeCache>,
}

impl OAuthService {
    // GET /oauth/authorize
    pub async fn authorize(&self, req: AuthorizeRequest) -> Result<AuthorizeResponse, AppError> {
        // 1. 验证 client_id
        let client = self.client_repo.find_by_client_id(&req.client_id).await?
            .ok_or(AppError::InvalidClient)?;

        // 2. 验证 redirect_uri
        if !self.client_repo.verify_redirect_uri(&req.client_id, &req.redirect_uri).await? {
            return Err(AppError::InvalidRedirectUri);
        }

        // 3. 生成授权码
        let code = generate_random_code();

        // 4. 保存授权码（5分钟过期）
        self.token_repo.save_auth_code(AuthorizationCode {
            code: code.clone(),
            client_id: req.client_id,
            user_id: req.user_id,
            redirect_uri: req.redirect_uri.clone(),
            scopes: req.scope.clone(),
            expires_at: chrono::Utc::now() + chrono::Duration::seconds(300),
        }).await?;

        // 5. 缓存授权码
        self.cache.set(&format!("code:{}", code), "valid", Some(300)).await;

        Ok(AuthorizeResponse { code, redirect_uri: req.redirect_uri })
    }

    // POST /oauth/token
    pub async fn token(&self, req: TokenRequest) -> Result<TokenResponse, AppError> {
        // 1. 验证授权码
        let auth_data = self.token_repo.consume_auth_code(&req.code).await?
            .ok_or(AppError::InvalidAuthCode)?;

        // 2. 验证客户端
        let client = self.client_repo.find_by_client_id(&auth_data.client_id).await?
            .ok_or(AppError::InvalidClient)?;

        if client.client_secret != req.client_secret {
            return Err(AppError::InvalidClient);
        }

        // 3. 生成 access_token
        let access_token = self.jwt_manager.generate_token(auth_data.user_id, 3600)?;
        let refresh_token = self.jwt_manager.generate_token(auth_data.user_id, 2592000)?;

        // 4. 保存 Token
        self.token_repo.save_access_token(AccessToken {
            token: access_token.clone(),
            user_id: auth_data.user_id,
            client_id: auth_data.client_id,
            scopes: auth_data.scopes.clone(),
            expires_at: chrono::Utc::now() + chrono::Duration::seconds(3600),
        }).await?;

        Ok(TokenResponse {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: 3600,
        })
    }
}
```

---

### 5.7 中间件 (api/middleware/)

**api/middleware/auth.rs:**
```rust
pub struct JwtAuth {
    jwt_manager: Arc<JwtManager>,
    cache: Arc<CompositeCache>,
}

impl<S, B> Transform<S, ServiceRequest> for JwtAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = JwtAuthMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(JwtAuthMiddleware {
            service,
            jwt_manager: self.jwt_manager.clone(),
            cache: self.cache.clone(),
        }))
    }
}

impl<S, B> Service<ServiceRequest> for JwtAuthMiddleware<S> {
    async fn call(&self, req: ServiceRequest) -> Result<Self::Response, Self::Error> {
        // 1. 提取 Token
        let token = extract_bearer_token(&req)?;

        // 2. 检查黑名单
        if self.cache.exists(&format!("blacklist:{}", token)).await {
            return Err(AppError::TokenExpired.into());
        }

        // 3. 验证 JWT
        let claims = self.jwt_manager.verify_token(token)?;

        // 4. 注入用户信息到请求
        req.extensions_mut().insert(claims);

        self.service.call(req).await
    }
}
```

**api/middleware/rate_limit.rs:**
```rust
pub struct RateLimiter {
    redis: Arc<RedisPool>,
    max_requests: u32,
    window: u64,  // 秒
}

impl RateLimiter {
    pub async fn check(&self, key: &str) -> Result<bool, AppError> {
        let redis_key = format!("ratelimit:{}", key);
        let mut conn = self.redis.get_connection().await?;

        // Redis INCR + EXPIRE 实现滑动窗口
        let count: u32 = conn.incr(&redis_key, 1).await?;
        if count == 1 {
            conn.expire(&redis_key, self.window as usize).await?;
        }

        Ok(count <= self.max_requests)
    }
}
```

---

### 5.8 启动流程 (runtime/startup.rs)

```rust
pub struct StartupContext {
    pub storage: Arc<SeaOrmStorage>,
    pub cache: Arc<CompositeCache>,
    pub jwt_manager: Arc<JwtManager>,
    pub config: AppConfig,
}

pub async fn prepare_server(config: AppConfig) -> Result<StartupContext, AppError> {
    // 1. 初始化 Rust-TLS
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install crypto provider");

    // 2. 初始化日志
    crate::system::logging::init_logging(&config.log);
    tracing::info!("FerrusGate-Lite v0.0.1 starting...");

    // 3. 初始化存储
    tracing::info!("Connecting to database: {}", config.database.url);
    let storage = Arc::new(SeaOrmStorage::new(&config.database).await?);

    // 4. 运行数据库迁移
    tracing::info!("Running database migrations...");
    migration::Migrator::up(&storage.db, None).await?;

    // 5. 初始化缓存
    tracing::info!("Initializing cache system...");
    let memory_cache = Arc::new(MemoryCache::new(config.cache.memory_cache_size));
    let redis_cache = Arc::new(RedisCache::new(&config.redis).await?);
    let cache = Arc::new(CompositeCache::new(memory_cache, redis_cache));

    // 6. 初始化 JWT 管理器
    let jwt_manager = Arc::new(JwtManager::new(config.auth.jwt_secret.clone()));

    tracing::info!("Server initialization complete");

    Ok(StartupContext {
        storage,
        cache,
        jwt_manager,
        config,
    })
}
```

**runtime/server.rs:**
```rust
pub async fn run_server(ctx: StartupContext) -> std::io::Result<()> {
    let bind_addr = format!("{}:{}", ctx.config.server.host, ctx.config.server.port);

    tracing::info!("Starting HTTP server on {}", bind_addr);

    HttpServer::new(move || {
        App::new()
            // 共享状态
            .app_data(web::Data::new(ctx.storage.clone()))
            .app_data(web::Data::new(ctx.cache.clone()))
            .app_data(web::Data::new(ctx.jwt_manager.clone()))

            // 中间件
            .wrap(middleware::Logger::default())
            .wrap(middleware::Cors::default())

            // 健康检查（无需认证）
            .service(web::scope("/health")
                .route("", web::get().to(health::health_check))
                .route("/ready", web::get().to(health::readiness))
                .route("/live", web::get().to(health::liveness))
            )

            // 认证 API（无需认证）
            .service(web::scope("/api/auth")
                .route("/register", web::post().to(auth_service::register))
                .route("/login", web::post().to(auth_service::login))
            )

            // OAuth2 授权端点
            .service(web::scope("/oauth")
                .route("/authorize", web::get().to(oauth_service::authorize))
                .route("/token", web::post().to(oauth_service::token))
            )

            // OIDC 端点
            .service(web::scope("/.well-known")
                .route("/openid-configuration", web::get().to(oidc_service::discovery))
                .route("/jwks.json", web::get().to(oidc_service::jwks))
            )
            .service(web::scope("/oauth")
                .route("/userinfo", web::get().to(oidc_service::userinfo)
                    .wrap(JwtAuth::new(ctx.jwt_manager.clone(), ctx.cache.clone())))
            )

            // 用户 API（需要认证）
            .service(web::scope("/api/user")
                .wrap(JwtAuth::new(ctx.jwt_manager.clone(), ctx.cache.clone()))
                .route("/me", web::get().to(user_service::get_profile))
                .route("/authorizations", web::get().to(user_service::list_authorizations))
                .route("/authorizations/{client_id}", web::delete().to(user_service::revoke_authorization))
            )
    })
    .bind(&bind_addr)?
    .run()
    .await
}
```

---

## 六、数据库设计

### 6.1 数据表结构

**users (用户表)**
```sql
CREATE TABLE users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username VARCHAR(50) UNIQUE NOT NULL,
    email VARCHAR(100) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX idx_users_username ON users(username);
CREATE INDEX idx_users_email ON users(email);
```

**oauth_clients (OAuth2 客户端)**
```sql
CREATE TABLE oauth_clients (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    client_id VARCHAR(255) UNIQUE NOT NULL,
    client_secret VARCHAR(255) NOT NULL,
    name VARCHAR(100) NOT NULL,
    redirect_uris TEXT NOT NULL,  -- JSON array
    allowed_scopes TEXT NOT NULL, -- JSON array
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX idx_clients_client_id ON oauth_clients(client_id);
```

**authorization_codes (授权码)**
```sql
CREATE TABLE authorization_codes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    code VARCHAR(255) UNIQUE NOT NULL,
    client_id VARCHAR(255) NOT NULL,
    user_id INTEGER NOT NULL,
    redirect_uri TEXT NOT NULL,
    scopes TEXT NOT NULL,          -- JSON array
    expires_at TIMESTAMP NOT NULL,
    used BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id),
    FOREIGN KEY (client_id) REFERENCES oauth_clients(client_id)
);
CREATE INDEX idx_codes_code ON authorization_codes(code);
CREATE INDEX idx_codes_expires_at ON authorization_codes(expires_at);
```

**access_tokens (访问令牌)**
```sql
CREATE TABLE access_tokens (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    token TEXT UNIQUE NOT NULL,
    token_type VARCHAR(20) DEFAULT 'Bearer',
    client_id VARCHAR(255) NOT NULL,
    user_id INTEGER NOT NULL,
    scopes TEXT NOT NULL,          -- JSON array
    expires_at TIMESTAMP NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id),
    FOREIGN KEY (client_id) REFERENCES oauth_clients(client_id)
);
CREATE INDEX idx_tokens_token ON access_tokens(token(100));
CREATE INDEX idx_tokens_expires_at ON access_tokens(expires_at);
```

**refresh_tokens (刷新令牌)**
```sql
CREATE TABLE refresh_tokens (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    token TEXT UNIQUE NOT NULL,
    access_token_id INTEGER NOT NULL,
    expires_at TIMESTAMP NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (access_token_id) REFERENCES access_tokens(id)
);
CREATE INDEX idx_refresh_tokens_token ON refresh_tokens(token(100));
```

---

## 七、API 接口设计

### 7.1 认证 API

| 端点 | 方法 | 说明 | 认证 |
|------|------|------|------|
| `/api/auth/register` | POST | 用户注册 | ❌ |
| `/api/auth/login` | POST | 用户登录 | ❌ |

**POST /api/auth/register**
```json
// Request
{
  "username": "user123",
  "email": "user@example.com",
  "password": "SecurePass123!"
}

// Response 201
{
  "user_id": 1,
  "message": "User created successfully"
}
```

**POST /api/auth/login**
```json
// Request
{
  "username": "user123",
  "password": "SecurePass123!"
}

// Response 200
{
  "access_token": "eyJhbGciOiJIUzI1NiIs...",
  "refresh_token": "eyJhbGciOiJIUzI1NiIs...",
  "token_type": "Bearer",
  "expires_in": 3600
}
```

---

### 7.2 OAuth2 API

| 端点 | 方法 | 说明 | 认证 |
|------|------|------|------|
| `/oauth/authorize` | GET | 授权请求 | ✅ (Session) |
| `/oauth/token` | POST | Token 换取 | ❌ |

**GET /oauth/authorize**
```
Query Parameters:
- response_type: code
- client_id: client_123
- redirect_uri: https://client.com/callback
- scope: openid profile email
- state: random_state_string

Response: 302 Redirect
Location: https://client.com/callback?code=AUTH_CODE&state=random_state_string
```

**POST /oauth/token**
```json
// Request
{
  "grant_type": "authorization_code",
  "code": "AUTH_CODE",
  "client_id": "client_123",
  "client_secret": "secret_xyz",
  "redirect_uri": "https://client.com/callback"
}

// Response 200
{
  "access_token": "eyJhbGciOiJIUzI1NiIs...",
  "refresh_token": "eyJhbGciOiJIUzI1NiIs...",
  "token_type": "Bearer",
  "expires_in": 3600,
  "id_token": "eyJhbGciOiJIUzI1NiIs..."  // OIDC
}
```

---

### 7.3 OIDC API

| 端点 | 方法 | 说明 | 认证 |
|------|------|------|------|
| `/.well-known/openid-configuration` | GET | Discovery 文档 | ❌ |
| `/.well-known/jwks.json` | GET | JWKS 公钥 | ❌ |
| `/oauth/userinfo` | GET | 用户信息 | ✅ (Bearer) |

**GET /oauth/userinfo**
```http
Authorization: Bearer eyJhbGciOiJIUzI1NiIs...

// Response 200
{
  "sub": "1",
  "name": "user123",
  "email": "user@example.com",
  "email_verified": true
}
```

---

### 7.4 用户管理 API

| 端点 | 方法 | 说明 | 认证 |
|------|------|------|------|
| `/api/user/me` | GET | 当前用户信息 | ✅ |
| `/api/user/authorizations` | GET | 已授权应用列表 | ✅ |
| `/api/user/authorizations/{client_id}` | DELETE | 撤销授权 | ✅ |

---

### 7.5 健康检查 API

| 端点 | 方法 | 说明 |
|------|------|------|
| `/health` | GET | 基础健康检查 |
| `/health/ready` | GET | 就绪检查（DB+Redis） |
| `/health/live` | GET | 存活检查 |

---

## 八、安全特性

1. **密码安全**：Bcrypt 加密存储
2. **Token 安全**：JWT 签名验证，支持黑名单
3. **HTTPS**：生产环境强制 HTTPS
4. **CORS**：可配置跨域策略
5. **限流**：基于 Redis 的 IP/用户级限流
6. **CSRF**：OAuth2 state 参数验证
7. **PKCE**：支持 OAuth2 PKCE 扩展（未来）

---

## 九、性能优化

1. **多层缓存**：L1(Moka) + L2(Redis) 减少 DB 查询
2. **连接池**：数据库和 Redis 连接池复用
3. **异步 I/O**：全异步处理，Tokio 多线程
4. **索引优化**：关键字段建立索引
5. **Token 缓存**：高频 Token 验证走缓存

---

## 十、可观测性

1. **结构化日志**：Tracing 框架，支持 JSON 输出
2. **健康检查**：Kubernetes 就绪/存活探针
3. **错误追踪**：统一错误类型和 HTTP 响应
4. **性能指标**：缓存命中率、请求延迟（未来）

---

## 十一、部署建议

**开发环境：**
```bash
cargo run
```

**生产环境（Docker）：**
```dockerfile
FROM rust:1.85 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/FerrusGate-Lite /usr/local/bin/
COPY config.toml /app/
WORKDIR /app
CMD ["FerrusGate-Lite"]
```

**环境变量覆盖：**
```bash
export JWT_SECRET="production-secret"
export DATABASE_URL="postgres://..."
export REDIS_URL="redis://..."
```

---

## 十二、未来扩展

- [ ] SAML 2.0 支持
- [ ] Passkey (FIDO2) 无密码登录
- [ ] 多因素认证 (MFA)
- [ ] 管理后台 UI
- [ ] 策略引擎（基于 Rego）
- [ ] 审计日志
- [ ] Prometheus 指标导出
- [ ] 多租户支持（完整版）

---

## 参考资料

- [OAuth 2.0 RFC 6749](https://datatracker.ietf.org/doc/html/rfc6749)
- [OpenID Connect Core 1.0](https://openid.net/specs/openid-connect-core-1_0.html)
- [JWT RFC 7519](https://datatracker.ietf.org/doc/html/rfc7519)
- [Actix-Web Documentation](https://actix.rs/)
- [SeaORM Documentation](https://www.sea-ql.org/SeaORM/)

---

**文档版本**: v0.1.0
**最后更新**: 2025-11-13
**作者**: AptS:1548 & AptS:1547
