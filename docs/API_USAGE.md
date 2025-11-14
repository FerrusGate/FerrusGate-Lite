# FerrusGate-Lite API 使用指南

## 导入 Swagger 文档

### 方式一：导入到 Postman

1. 打开 Postman
2. 点击左上角 **Import** 按钮
3. 选择 **File** 标签
4. 选择 `docs/swagger.yaml` 文件
5. 点击 **Import** 完成导入

导入后会自动生成一个完整的 Collection，包含所有 21 个 API 端点。

### 方式二：导入到 Apifox

1. 打开 Apifox
2. 选择项目后，点击顶部菜单 **快捷导入**
3. 选择 **导入数据** → **OpenAPI/Swagger**
4. 选择 `docs/swagger.yaml` 文件
5. 配置导入选项后点击 **确认导入**

### 方式三：在线预览

可以使用 Swagger Editor 在线预览和测试：

1. 访问 https://editor.swagger.io/
2. 将 `swagger.yaml` 文件内容粘贴到左侧编辑器
3. 右侧会自动渲染 API 文档

## API 端点概览

### 🏥 健康检查（无需认证）

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/health` | 基础健康检查 |
| GET | `/health/ready` | 就绪探针（检查数据库、缓存） |
| GET | `/health/live` | 存活探针 |

### 🔐 用户认证（无需认证）

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/api/auth/register` | 用户注册 |
| POST | `/api/auth/login` | 用户登录 |
| POST | `/api/auth/verify-invite` | 验证邀请码 |

### 🔑 OAuth2 & OIDC

| 方法 | 路径 | 认证 | 说明 |
|------|------|------|------|
| GET | `/oauth/authorize` | ❌ | OAuth2 授权请求 |
| POST | `/oauth/token` | ❌ | 获取 Access Token |
| GET | `/oauth/userinfo` | ✅ JWT | 获取用户信息 |
| GET | `/.well-known/openid-configuration` | ❌ | OIDC 发现文档 |
| GET | `/.well-known/jwks.json` | ❌ | JWKS 公钥 |

### 👤 用户 API（需要 JWT）

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/api/user/me` | 获取当前用户信息 |
| GET | `/api/user/authorizations` | 获取已授权应用列表 |
| DELETE | `/api/user/authorizations/{client_id}` | 撤销授权 |

### ⚙️ 管理员 API - 设置（需要管理员权限）

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/api/admin/settings/registration` | 获取注册配置 |
| PUT | `/api/admin/settings/registration` | 更新注册配置 |
| GET | `/api/admin/settings/audit-logs` | 获取审计日志 |

### 🎟️ 管理员 API - 邀请码（需要管理员权限）

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/api/admin/invites` | 创建邀请码 |
| GET | `/api/admin/invites` | 列出所有邀请码 |
| GET | `/api/admin/invites/stats` | 获取邀请码统计 |
| DELETE | `/api/admin/invites/{code}` | 撤销邀请码 |

## 快速开始

### 1. 启动服务

```bash
cargo run
```

默认服务地址：`http://127.0.0.1:8080`

### 2. 注册用户

**请求示例：**

```bash
curl -X POST http://127.0.0.1:8080/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "username": "testuser",
    "email": "test@example.com",
    "password": "SecurePass123!"
  }'
```

**响应示例：**

```json
{
  "user_id": 1,
  "message": "User created successfully"
}
```

### 3. 登录获取 Token

**请求示例：**

```bash
curl -X POST http://127.0.0.1:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "username": "testuser",
    "password": "SecurePass123!"
  }'
```

**响应示例：**

```json
{
  "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "refresh_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "token_type": "Bearer",
  "expires_in": 3600
}
```

### 4. 使用 Token 访问受保护端点

**请求示例：**

```bash
curl -X GET http://127.0.0.1:8080/api/user/me \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
```

**响应示例：**

```json
{
  "id": 1,
  "username": "testuser",
  "email": "test@example.com",
  "created_at": "2024-01-15T08:30:00Z"
}
```

## 认证说明

### JWT Bearer Token

大部分 API 端点需要 JWT 认证，在 HTTP Header 中添加：

```
Authorization: Bearer <your_access_token>
```

### 权限级别

系统支持基于角色的权限控制（RBAC）：

- **匿名访问**：健康检查、注册、登录、OIDC 发现等公开端点
- **普通用户**（`role: user`）：需要 JWT，可访问 `/api/user/*` 端点
- **管理员**（`role: admin`）：需要 JWT 且角色为 admin，可访问 `/api/admin/*` 端点

## OAuth2 授权流程

### 授权码模式（Authorization Code Flow）

1. **请求授权码**

```
GET /oauth/authorize?response_type=code&client_id=YOUR_CLIENT_ID&redirect_uri=https://example.com/callback&scope=openid%20profile%20email&state=random_state
```

2. **用户同意授权后重定向**

```
https://example.com/callback?code=AUTH_CODE&state=random_state
```

3. **授权码换取 Token**

```bash
curl -X POST http://127.0.0.1:8080/oauth/token \
  -H "Content-Type: application/json" \
  -d '{
    "grant_type": "authorization_code",
    "code": "AUTH_CODE",
    "client_id": "YOUR_CLIENT_ID",
    "client_secret": "YOUR_CLIENT_SECRET",
    "redirect_uri": "https://example.com/callback"
  }'
```

4. **使用 Access Token 获取用户信息**

```bash
curl -X GET http://127.0.0.1:8080/oauth/userinfo \
  -H "Authorization: Bearer ACCESS_TOKEN"
```

## 管理员操作示例

### 创建邀请码

```bash
curl -X POST http://127.0.0.1:8080/api/admin/invites \
  -H "Authorization: Bearer ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "max_uses": 5,
    "expires_in_hours": 168
  }'
```

### 更新注册配置

```bash
curl -X PUT http://127.0.0.1:8080/api/admin/settings/registration \
  -H "Authorization: Bearer ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "allow_registration": true,
    "allowed_email_domains": ["example.com"],
    "min_username_length": 3,
    "max_username_length": 32,
    "min_password_length": 8,
    "password_require_uppercase": true,
    "password_require_lowercase": true,
    "password_require_numbers": true,
    "password_require_special": true,
    "require_invite_code": true
  }'
```

### 查看审计日志

```bash
curl -X GET "http://127.0.0.1:8080/api/admin/settings/audit-logs?limit=50&config_key=registration_config" \
  -H "Authorization: Bearer ADMIN_TOKEN"
```

## 错误处理

所有错误响应遵循统一格式：

```json
{
  "error": "ErrorType",
  "message": "Detailed error message"
}
```

### 常见错误码

| HTTP 状态码 | 错误类型 | 说明 |
|------------|---------|------|
| 400 | BadRequest | 请求参数错误 |
| 401 | Unauthorized | 未认证或 Token 无效 |
| 403 | Forbidden | 权限不足 |
| 404 | NotFound | 资源不存在 |
| 500 | InternalServerError | 服务器内部错误 |

## 注意事项

1. **Token 过期**：Access Token 默认 1 小时过期，Refresh Token 默认 30 天过期
2. **密码安全**：系统使用 Argon2 算法哈希密码
3. **邀请码格式**：格式为 `INV-XXXXXXXXXXXX`（12 位大写字母和数字）
4. **邮箱域名限制**：管理员可配置允许注册的邮箱域名白名单
5. **黑名单机制**：撤销的 Token 会被加入黑名单（基于 Redis/内存缓存）

## 开发建议

### 使用 Postman/Apifox 环境变量

建议设置以下环境变量：

```
BASE_URL = http://127.0.0.1:8080
ACCESS_TOKEN = <登录后自动填充>
```

在请求中使用：

- URL: `{{BASE_URL}}/api/user/me`
- Header: `Authorization: Bearer {{ACCESS_TOKEN}}`

### 自动刷新 Token

可以在 Postman 的 Pre-request Script 中添加自动刷新逻辑：

```javascript
const token = pm.environment.get("ACCESS_TOKEN");
// 检查 token 是否过期，过期则自动刷新
// （需要配合 refresh_token 实现）
```

## 更多资源

- **项目仓库**：查看源码了解实现细节
- **配置文件**：`config.yaml` - 服务器、数据库、认证等配置
- **数据库迁移**：`migration/` - 数据库结构定义
- **实体模型**：`src/storage/entities/` - 数据库实体定义

## 问题反馈

如发现 API 文档有误或有改进建议，请提交 Issue 或 Pull Request。
