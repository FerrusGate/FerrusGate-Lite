#[cfg(test)]
mod tests {
    use super::super::backend::SeaOrmBackend;
    use crate::cache::{CompositeCache, MemoryCache};
    use crate::config::RegistrationConfig;
    use sea_orm::{Database, DatabaseConnection};
    use std::sync::Arc;

    /// 创建测试用的内存数据库
    async fn setup_test_db() -> Arc<DatabaseConnection> {
        let db = Database::connect("sqlite::memory:")
            .await
            .expect("Failed to create test database");

        // 运行 migrations
        use crate::storage::run_migrations;
        run_migrations(&db).await.expect("Failed to run migrations");

        Arc::new(db)
    }

    /// 创建测试用缓存
    fn create_test_cache() -> Arc<CompositeCache> {
        let l1 = Arc::new(MemoryCache::new(1000));
        let l2 = Arc::new(MemoryCache::new(1000));
        Arc::new(CompositeCache::new(l1, l2))
    }

    /// 创建测试用户
    async fn create_test_user(backend: &SeaOrmBackend) -> i64 {
        use crate::storage::UserRepository;
        let user = backend
            .create("testuser", "test@example.com", "hashedpassword")
            .await
            .expect("Failed to create test user");
        user.id
    }

    #[tokio::test]
    async fn test_registration_config_cache() {
        // 1. 设置
        let db = setup_test_db().await;
        let cache = create_test_cache();
        let backend = SeaOrmBackend::with_cache(db, cache.clone());

        // 2. 首次读取配置（应该从数据库读取）
        let config1 = backend
            .get_registration_config()
            .await
            .expect("Failed to get config");
        assert_eq!(config1.allow_registration, true);

        // 3. 再次读取配置（应该从缓存读取）
        let config2 = backend
            .get_registration_config()
            .await
            .expect("Failed to get config");
        assert_eq!(config2.allow_registration, true);

        // 4. 验证缓存键存在
        let cached = cache.get("config:registration").await;
        assert!(cached.is_some(), "Config should be cached");
    }

    #[tokio::test]
    async fn test_registration_config_cache_invalidation() {
        // 1. 设置
        let db = setup_test_db().await;
        let cache = create_test_cache();
        let backend = SeaOrmBackend::with_cache(db, cache.clone());
        let user_id = create_test_user(&backend).await;

        // 2. 读取配置（填充缓存）
        let _ = backend.get_registration_config().await;

        // 3. 更新配置
        let mut new_config = RegistrationConfig::default();
        new_config.allow_registration = false;
        new_config.min_password_length = 12;

        backend
            .update_registration_config(&new_config, user_id)
            .await
            .expect("Failed to update config");

        // 4. 验证缓存已清除
        let cached = cache.get("config:registration").await;
        assert!(cached.is_none(), "Cache should be invalidated after update");

        // 5. 验证新配置生效
        let updated_config = backend
            .get_registration_config()
            .await
            .expect("Failed to get updated config");
        assert_eq!(updated_config.allow_registration, false);
        assert_eq!(updated_config.min_password_length, 12);
    }

    #[tokio::test]
    async fn test_config_audit_log() {
        // 1. 设置
        let db = setup_test_db().await;
        let backend = SeaOrmBackend::new(db);
        let user_id = create_test_user(&backend).await;

        // 2. 更新配置（应该自动记录审计日志）
        let mut new_config = RegistrationConfig::default();
        new_config.allow_registration = false;

        backend
            .update_registration_config(&new_config, user_id)
            .await
            .expect("Failed to update config");

        // 3. 查询审计日志
        let logs = backend
            .get_config_audit_logs(Some(10))
            .await
            .expect("Failed to get audit logs");

        // 4. 验证日志内容
        assert_eq!(logs.len(), 1, "Should have one audit log entry");
        assert_eq!(logs[0].config_key, "registration_config");
        assert_eq!(logs[0].changed_by, user_id);
        assert_eq!(logs[0].change_type, Some("update".to_string()));
        assert!(logs[0].old_value.is_some(), "Should have old value");
        assert!(logs[0].new_value.is_some(), "Should have new value");
    }

    #[tokio::test]
    async fn test_invite_code_stats() {
        use chrono::{Duration, Utc};

        // 1. 设置
        let db = setup_test_db().await;
        let backend = SeaOrmBackend::new(db);
        let user_id = create_test_user(&backend).await;

        // 2. 创建几个邀请码
        // 活跃的邀请码
        backend
            .create_invite_code("CODE1", user_id, 5, None)
            .await
            .expect("Failed to create invite 1");

        // 已用完的邀请码
        backend
            .create_invite_code("CODE2", user_id, 1, None)
            .await
            .expect("Failed to create invite 2");
        backend
            .verify_and_use_invite_code("CODE2", user_id)
            .await
            .expect("Failed to use invite 2");

        // 已过期的邀请码
        let past_time = Utc::now() - Duration::days(1);
        backend
            .create_invite_code("CODE3", user_id, 3, Some(past_time))
            .await
            .expect("Failed to create invite 3");

        // 3. 获取统计
        let stats = backend
            .get_invite_stats()
            .await
            .expect("Failed to get stats");

        // 4. 验证统计结果
        assert_eq!(stats.total_count, 3, "Should have 3 invite codes");
        assert_eq!(stats.active_count, 1, "Should have 1 active code");
        assert_eq!(stats.fully_used_count, 1, "Should have 1 fully used code");
        assert_eq!(stats.expired_count, 1, "Should have 1 expired code");
        assert_eq!(stats.total_uses, 1, "Should have 1 total use");
        assert_eq!(stats.total_capacity, 9, "Total capacity should be 5+1+3=9");
    }

    #[tokio::test]
    async fn test_invite_code_verification() {
        // 1. 设置
        let db = setup_test_db().await;
        let backend = SeaOrmBackend::new(db);
        let user_id = create_test_user(&backend).await;

        // 2. 创建只允许使用 1 次的邀请码
        backend
            .create_invite_code("TESTCODE", user_id, 1, None)
            .await
            .expect("Failed to create invite code");

        // 3. 第一次使用（应该成功）
        backend
            .verify_and_use_invite_code("TESTCODE", user_id)
            .await
            .expect("First use should succeed");

        // 4. 验证使用计数
        let invite = backend
            .find_invite_code("TESTCODE")
            .await
            .expect("Failed to find invite")
            .expect("Invite should exist");
        assert_eq!(
            invite.used_count, 1,
            "Used count should be 1 after first use"
        );
        assert_eq!(invite.used_by, Some(user_id), "Should record first user");

        // 5. 第二次使用（应该失败，已用完）
        use crate::storage::UserRepository;
        let user2 = backend
            .create("testuser2", "test2@example.com", "hashedpassword2")
            .await
            .expect("Failed to create test user 2");

        let result = backend
            .verify_and_use_invite_code("TESTCODE", user2.id)
            .await;
        assert!(
            result.is_err(),
            "Second use should fail because invite is already fully used"
        );
    }

    #[tokio::test]
    async fn test_expired_invite_code() {
        use chrono::{Duration, Utc};

        // 1. 设置
        let db = setup_test_db().await;
        let backend = SeaOrmBackend::new(db);
        let user_id = create_test_user(&backend).await;

        // 2. 创建已过期的邀请码
        let past_time = Utc::now() - Duration::hours(1);
        backend
            .create_invite_code("EXPIRED", user_id, 5, Some(past_time))
            .await
            .expect("Failed to create expired invite");

        // 3. 尝试使用过期邀请码（应该失败）
        let result = backend.verify_and_use_invite_code("EXPIRED", user_id).await;
        assert!(result.is_err(), "Using expired invite should fail");
    }
}
