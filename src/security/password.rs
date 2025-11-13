use crate::errors::AppError;

pub struct PasswordManager;

impl PasswordManager {
    /// 对密码进行哈希加密
    pub fn hash_password(password: &str) -> Result<String, AppError> {
        bcrypt::hash(password, bcrypt::DEFAULT_COST)
            .map_err(|e| AppError::Internal(format!("Password hash failed: {}", e)))
    }

    /// 验证密码是否匹配
    pub fn verify_password(password: &str, hash: &str) -> Result<bool, AppError> {
        bcrypt::verify(password, hash)
            .map_err(|e| AppError::Internal(format!("Password verify failed: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hash_and_verify() {
        let password = "MySecurePassword123!";
        let hash = PasswordManager::hash_password(password).unwrap();

        assert!(PasswordManager::verify_password(password, &hash).unwrap());
        assert!(!PasswordManager::verify_password("WrongPassword", &hash).unwrap());
    }
}
