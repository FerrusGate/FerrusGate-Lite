# 🚀 FerrusGate-Lite 快速开始

FerrusGate-Lite 的轻量级 OAuth2/OIDC 身份认证网关已经实现完成！

## 📦 已实现的功能

### ✅ 核心功能
- **用户认证** - 注册和登录
- **OAuth2 授权码流程** - 完整的授权码模式
- **OIDC 支持** - Discovery、JWKS、UserInfo
- **JWT 认证** - Token 生成和验证
- **用户管理 API** - 用户信息查询
- **健康检查** - 支持 Kubernetes 探针
- **多层缓存** - Moka (L1) + Redis (L2)
- **数据库迁移** - SeaORM 自动迁移

### 🔧 技术架构
- Actix-Web 4.11 - 高性能异步 HTTP
- SeaORM 2.0-rc - 异步 ORM
- Redis - 分布式缓存
- SQLite/PostgreSQL/MySQL - 可切换数据库
- Tokio - 异步运行时

## 🏃 运行项目

### 1. 安装依赖

确保你已安装：
- Rust 1.85+ (支持 Edition 2024)
- Redis (可选，失败会降级到纯内存缓存)

### 2. 启动服务器

```bash
# 首次运行会自动创建数据库和执行迁移
cargo run
```

服务器默认监听: `http://127.0.0.1:8080`

### 3. 初始化种子数据

在另一个终端执行：

```bash
./seed.sh
```

这会创建：
- **测试用户**:
  - `testuser` / `password123`
  - `admin` / `password123`

- **OAuth 客户端**:
  - `test_client_123` / `test_secret_456`
  - `demo_app` / `demo_secret_xyz`

## 🧪 测试 API

### 健康检查
```bash
curl http://localhost:8080/health
```

### 用户注册
```bash
curl -X POST http://localhost:8080/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{"username":"newuser","email":"new@example.com","password":"password123"}'
```

### 用户登录
```bash
curl -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"testuser","password":"password123"}'
```

会返回 `access_token` 和 `refresh_token`。

### OAuth2 授权流程

#### 1. 获取授权码
```bash
# 在浏览器访问
http://localhost:8080/oauth/authorize?response_type=code&client_id=test_client_123&redirect_uri=http://localhost:3000/callback&scope=openid%20profile%20email&state=random_state
```

会重定向到 `http://localhost:3000/callback?code=XXXXX&state=random_state`

#### 2. 换取 Access Token
```bash
curl -X POST http://localhost:8080/oauth/token \
  -H "Content-Type: application/json" \
  -d '{
    "grant_type":"authorization_code",
    "code":"你的授权码",
    "client_id":"test_client_123",
    "client_secret":"test_secret_456",
    "redirect_uri":"http://localhost:3000/callback"
  }'
```

### OIDC Discovery
```bash
curl http://localhost:8080/.well-known/openid-configuration
```

### UserInfo (需要 Token)
```bash
curl http://localhost:8080/oauth/userinfo \
  -H "Authorization: Bearer YOUR_ACCESS_TOKEN"
```

### 用户信息查询 (需要 Token)
```bash
curl http://localhost:8080/api/user/me \
  -H "Authorization: Bearer YOUR_ACCESS_TOKEN"
```

## 📝 API 端点汇总

| 端点 | 方法 | 认证 | 说明 |
|------|------|------|------|
| `/health` | GET | ❌ | 基础健康检查 |
| `/health/ready` | GET | ❌ | 就绪检查（DB+Redis） |
| `/health/live` | GET | ❌ | 存活检查 |
| `/api/auth/register` | POST | ❌ | 用户注册 |
| `/api/auth/login` | POST | ❌ | 用户登录 |
| `/oauth/authorize` | GET | ⚠️ | OAuth2 授权页面 |
| `/oauth/token` | POST | ❌ | 换取 Token |
| `/oauth/userinfo` | GET | ✅ | OIDC UserInfo |
| `/.well-known/openid-configuration` | GET | ❌ | OIDC Discovery |
| `/.well-known/jwks.json` | GET | ❌ | JWKS 公钥 |
| `/api/user/me` | GET | ✅ | 当前用户信息 |
| `/api/user/authorizations` | GET | ✅ | 已授权应用列表 |
| `/api/user/authorizations/{client_id}` | DELETE | ✅ | 撤销授权 |

## ⚙️ 配置

编辑 `config.toml`:

```toml
[server]
host = "127.0.0.1"
port = 8080

[database]
url = "sqlite://ferrusgate.db?mode=rwc"  # 或 PostgreSQL/MySQL

[redis]
url = "redis://127.0.0.1:6379"

[auth]
jwt_secret = "your-secret-key-change-me-in-production"
access_token_expire = 3600       # 1小时
refresh_token_expire = 2592000   # 30天

[log]
level = "info"  # trace, debug, info, warn, error
format = "pretty"  # pretty 或 json
```

## 🔒 生产环境注意事项

⚠️ **当前实现的简化部分（需要生产化）：**

1. **OAuth2 授权页面**: 当前直接生成授权码，实际应该：
   - 验证用户登录状态（Session/Cookie）
   - 显示授权确认页面
   - 用户同意后才生成授权码

2. **JWT 密钥**: 使用环境变量或密钥管理服务
3. **HTTPS**: 生产环境必须使用 HTTPS
4. **JWKS**: 切换到 RS256 非对称加密
5. **数据库**: 建议使用 PostgreSQL

## 📚 下一步

- [ ] 实现真实的用户会话管理
- [ ] 添加 OAuth2 授权确认页面
- [ ] 实现 Refresh Token 刷新
- [ ] 添加 PKCE 支持
- [ ] 实现限流中间件
- [ ] 添加审计日志
- [ ] 单元测试和集成测试

## 🎉 完成状态

所有核心功能已实现并通过编译！项目可以正常运行。

---

**作者**: AptS:1547 & AptS:1548
**版本**: v0.0.1
