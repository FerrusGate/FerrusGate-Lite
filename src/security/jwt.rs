use crate::errors::AppError;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,                // user_id
    pub exp: i64,                   // 过期时间戳
    pub iat: i64,                   // 签发时间戳
    pub scope: Option<Vec<String>>, // 权限范围（可选）
    pub role: String,               // 用户角色
}

pub struct JwtManager {
    secret: String,
}

impl JwtManager {
    pub fn new(secret: String) -> Self {
        Self { secret }
    }

    /// 生成 JWT Token
    pub fn generate_token(
        &self,
        user_id: i64,
        expire_in: i64,
        scope: Option<Vec<String>>,
        role: &str,
    ) -> Result<String, AppError> {
        let now = chrono::Utc::now().timestamp();
        let claims = Claims {
            sub: user_id.to_string(),
            exp: now + expire_in,
            iat: now,
            scope,
            role: role.to_string(),
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
        .map_err(|e| AppError::Internal(format!("JWT encode failed: {}", e)))
    }

    /// 验证并解析 Token
    pub fn verify_token(&self, token: &str) -> Result<Claims, AppError> {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = true;

        decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &validation,
        )
        .map(|data| data.claims)
        .map_err(|e| match e.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => AppError::TokenExpired,
            _ => AppError::InvalidToken,
        })
    }

    /// 提取 Token 中的 user_id
    pub fn extract_user_id(&self, token: &str) -> Result<i64, AppError> {
        let claims = self.verify_token(token)?;
        claims
            .sub
            .parse::<i64>()
            .map_err(|_| AppError::InvalidToken)
    }

    /// 获取密钥（用于 ID Token 生成等场景）
    pub fn secret(&self) -> &str {
        &self.secret
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jwt_generate_and_verify() {
        let manager = JwtManager::new("test-secret-key-at-least-32-characters-long".to_string());
        let token = manager.generate_token(123, 3600, None, "user").unwrap();

        let claims = manager.verify_token(&token).unwrap();
        assert_eq!(claims.sub, "123");
        assert_eq!(claims.role, "user");

        let user_id = manager.extract_user_id(&token).unwrap();
        assert_eq!(user_id, 123);
    }

    #[test]
    fn test_jwt_expired_token() {
        let manager = JwtManager::new("test-secret-key-at-least-32-characters-long".to_string());
        // 过期时间设为 -1 秒（已过期）
        let token = manager.generate_token(123, -1, None, "user").unwrap();

        let result = manager.verify_token(&token);
        assert!(matches!(result, Err(AppError::TokenExpired)));
    }
}
