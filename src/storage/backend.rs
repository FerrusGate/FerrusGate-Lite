use sea_orm::DatabaseConnection;
use std::sync::Arc;

use crate::cache::CompositeCache;

/// SeaORM 存储后端
pub struct SeaOrmBackend {
    pub(crate) db: Arc<DatabaseConnection>,
    pub(crate) cache: Option<Arc<CompositeCache>>,
}

impl SeaOrmBackend {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db, cache: None }
    }

    pub fn with_cache(db: Arc<DatabaseConnection>, cache: Arc<CompositeCache>) -> Self {
        Self {
            db,
            cache: Some(cache),
        }
    }
}
