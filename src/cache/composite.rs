use async_trait::async_trait;
use std::sync::Arc;

use crate::cache::traits::Cache;

/// 组合缓存（L1 内存 + L2 Redis）
pub struct CompositeCache {
    l1: Arc<dyn Cache>,  // 内存缓存
    l2: Arc<dyn Cache>,  // Redis 缓存
}

impl CompositeCache {
    pub fn new(l1: Arc<dyn Cache>, l2: Arc<dyn Cache>) -> Self {
        Self { l1, l2 }
    }

    /// 获取缓存（先查 L1，未命中查 L2，并回填 L1）
    pub async fn get(&self, key: &str) -> Option<String> {
        // 先查 L1
        if let Some(value) = self.l1.get(key).await {
            tracing::debug!("Cache L1 hit: {}", key);
            return Some(value);
        }

        // 再查 L2
        if let Some(value) = self.l2.get(key).await {
            tracing::debug!("Cache L2 hit: {}", key);
            // 回填 L1
            self.l1.set(key, value.clone(), None).await;
            return Some(value);
        }

        tracing::debug!("Cache miss: {}", key);
        None
    }

    /// 设置缓存（同时写入 L1 和 L2）
    pub async fn set(&self, key: &str, value: String, ttl: Option<u64>) {
        self.l1.set(key, value.clone(), ttl).await;
        self.l2.set(key, value, ttl).await;
    }

    /// 删除缓存（同时删除 L1 和 L2）
    pub async fn delete(&self, key: &str) {
        self.l1.delete(key).await;
        self.l2.delete(key).await;
    }

    /// 检查键是否存在（先查 L1，再查 L2）
    pub async fn exists(&self, key: &str) -> bool {
        self.l1.exists(key).await || self.l2.exists(key).await
    }

    /// 清空所有缓存
    pub async fn clear(&self) {
        self.l1.clear().await;
        self.l2.clear().await;
    }
}

#[async_trait]
impl Cache for CompositeCache {
    async fn get(&self, key: &str) -> Option<String> {
        self.get(key).await
    }

    async fn set(&self, key: &str, value: String, ttl: Option<u64>) {
        self.set(key, value, ttl).await;
    }

    async fn delete(&self, key: &str) {
        self.delete(key).await;
    }

    async fn exists(&self, key: &str) -> bool {
        self.exists(key).await
    }

    async fn clear(&self) {
        self.clear().await;
    }
}
