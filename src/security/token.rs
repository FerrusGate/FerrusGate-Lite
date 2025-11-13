use rand::Rng;
use rand::distributions::Alphanumeric;

/// 生成随机字符串
pub fn generate_random_string(length: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}

/// 生成授权码（32字符）
pub fn generate_auth_code() -> String {
    generate_random_string(32)
}

/// 生成客户端 Secret（48字符）
pub fn generate_client_secret() -> String {
    generate_random_string(48)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_auth_code() {
        let code = generate_auth_code();
        assert_eq!(code.len(), 32);
        assert!(code.chars().all(|c| c.is_alphanumeric()));
    }

    #[test]
    fn test_generate_client_secret() {
        let secret = generate_client_secret();
        assert_eq!(secret.len(), 48);
    }
}
