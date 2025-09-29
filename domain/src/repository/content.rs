use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::model::content::ContentEntity;
use common::types::BoxError;

#[rustfmt::skip]
#[async_trait]
pub trait ContentRepository: Send + Sync {
    async fn create(&self, entity: &ContentEntity) -> Result<ContentEntity, BoxError>;
    async fn find(&self, id: &str, date: &DateTime<Utc>) -> Result<Option<ContentEntity>, BoxError>;
    async fn remove(&self, id: &str) -> Result<u64, BoxError>;
}
