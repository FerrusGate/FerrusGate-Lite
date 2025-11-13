use async_trait::async_trait;

/// 缓存特征
#[async_trait]
pub trait Cache: Send + Sync {
    /// 获取缓存值
    async fn get(&self, key: &str) -> Option<String>;

    /// 设置缓存值
    async fn set(&self, key: &str, value: String, ttl: Option<u64>);

    /// 删除缓存
    async fn delete(&self, key: &str);

    /// 检查键是否存在
    async fn exists(&self, key: &str) -> bool;

    /// 清空所有缓存
    async fn clear(&self);
}
