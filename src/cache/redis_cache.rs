use async_trait::async_trait;
use redis::aio::MultiplexedConnection;
use redis::{AsyncCommands, Client};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::cache::traits::Cache;
use crate::config::RedisConfig;
use crate::errors::AppError;

/// Redis 缓存实现
pub struct RedisCache {
    conn: Arc<Mutex<MultiplexedConnection>>,
}

impl RedisCache {
    pub async fn new(config: &RedisConfig) -> Result<Self, AppError> {
        let client = Client::open(config.url.as_str())?;
        let conn = client.get_multiplexed_async_connection().await?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }
}

#[async_trait]
impl Cache for RedisCache {
    async fn get(&self, key: &str) -> Option<String> {
        let mut conn = self.conn.lock().await;
        conn.get(key).await.ok()
    }

    async fn set(&self, key: &str, value: String, ttl: Option<u64>) {
        let mut conn = self.conn.lock().await;
        if let Some(ttl_secs) = ttl {
            let _: Result<(), redis::RedisError> = conn.set_ex(key, value, ttl_secs).await;
        } else {
            let _: Result<(), redis::RedisError> = conn.set(key, value).await;
        }
    }

    async fn delete(&self, key: &str) {
        let mut conn = self.conn.lock().await;
        let _: Result<(), redis::RedisError> = conn.del(key).await;
    }

    async fn exists(&self, key: &str) -> bool {
        let mut conn = self.conn.lock().await;
        conn.exists(key).await.unwrap_or(false)
    }

    async fn clear(&self) {
        // Redis FLUSHDB - 清空当前数据库（慎用！）
        let mut conn = self.conn.lock().await;
        let _: Result<(), redis::RedisError> = redis::cmd("FLUSHDB").query_async(&mut *conn).await;
    }
}
