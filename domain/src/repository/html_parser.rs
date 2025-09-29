use async_trait::async_trait;

use crate::model::content::ContentEntity;
use common::types::BoxError;

#[rustfmt::skip]
#[async_trait]
pub trait HtmlParserRepository: Send + Sync {
    async fn create(&self, entity: &ContentEntity) -> Result<String, BoxError>;
    async fn remove(&self, id: &str) -> Result<u64, BoxError>;
}
