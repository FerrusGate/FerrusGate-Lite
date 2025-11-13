use async_trait::async_trait;
use moka::future::Cache as MokaCache;
use std::sync::Arc;
use std::time::Duration;

use crate::cache::traits::Cache;

/// 内存缓存实现（基于 Moka）
pub struct MemoryCache {
    cache: Arc<MokaCache<String, String>>,
}

impl MemoryCache {
    pub fn new(max_capacity: u64) -> Self {
        let cache = MokaCache::builder()
            .max_capacity(max_capacity)
            .time_to_live(Duration::from_secs(300)) // 默认 5 分钟过期
            .build();

        Self {
            cache: Arc::new(cache),
        }
    }
}

#[async_trait]
impl Cache for MemoryCache {
    async fn get(&self, key: &str) -> Option<String> {
        self.cache.get(key).await
    }

    async fn set(&self, key: &str, value: String, _ttl: Option<u64>) {
        // Moka 的 TTL 在创建 Cache 时统一设置，这里简化为直接 insert
        // 如果需要per-key TTL，需要使用不同的方案
        self.cache.insert(key.to_string(), value).await;
    }

    async fn delete(&self, key: &str) {
        self.cache.invalidate(key).await;
    }

    async fn exists(&self, key: &str) -> bool {
        self.cache.contains_key(key)
    }

    async fn clear(&self) {
        self.cache.invalidate_all();
    }
}
