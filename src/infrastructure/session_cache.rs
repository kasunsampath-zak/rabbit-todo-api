use moka::future::Cache;
use std::sync::Arc;
use std::time::Duration;

use crate::domain::models::Session;

#[derive(Clone)]
pub struct SessionCache {
    cache: Arc<Cache<String, Session>>,
}

impl SessionCache {
    pub fn new(max_capacity: u64, ttl_seconds: u64) -> Self {
        let cache = Cache::builder()
            .max_capacity(max_capacity)
            .time_to_live(Duration::from_secs(ttl_seconds))
            .build();

        Self {
            cache: Arc::new(cache),
        }
    }

    pub async fn get(&self, session_id: &str) -> Option<Session> {
        self.cache.get(session_id).await
    }

    pub async fn set(&self, session_id: String, session: Session) {
        self.cache.insert(session_id, session).await;
    }

    pub async fn remove(&self, session_id: &str) {
        self.cache.invalidate(session_id).await;
    }
}
