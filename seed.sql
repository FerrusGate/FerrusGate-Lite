-- FerrusGate-Lite 数据库种子数据
-- 用于测试和开发

-- 插入测试用户
-- 密码: password123 (已通过 argon2 加密)
INSERT INTO users (username, email, password_hash, role, created_at, updated_at)
VALUES
    ('admin', 'admin@example.com', '$argon2i$v=19$m=16,t=2,p=1$MTk3Mzl5c2Fk$PyyOvH/WHwhJAhmUyTOtkw', 'admin', datetime('now'), datetime('now')),
    ('testuser', 'test@example.com', '$argon2i$v=19$m=16,t=2,p=1$MTk3Mzl5c2Fk$PyyOvH/WHwhJAhmUyTOtkw', 'user', datetime('now'), datetime('now'));

-- 插入测试 OAuth 客户端
INSERT INTO o_auth_clients (client_id, client_secret, name, redirect_uris, allowed_scopes, created_at)
VALUES
    (
        'test_client_123',
        'test_secret_456',
        'Test Application',
        '["http://localhost:3000/callback", "http://localhost:8080/callback"]',
        '["openid", "profile", "email", "read", "write"]',
        datetime('now')
    ),
    (
        'demo_app',
        'demo_secret_xyz',
        'Demo Application',
        '["http://localhost:4000/auth/callback"]',
        '["openid", "profile", "email"]',
        datetime('now')
    );
