use async_trait::async_trait;

#[allow(unused_imports)]
use crate::model::{
    content::ContentEntity,
    search_engine::{SearchParams, SearchResult},
};
use common::types::BoxError;

#[async_trait]
pub trait SearchEngineRepository: Send + Sync {
    async fn create(&self, params: &ContentEntity) -> Result<ContentEntity, BoxError>;
    async fn edit(&self, params: &ContentEntity) -> Result<Option<ContentEntity>, BoxError>;
    async fn remove(&self, id: &str) -> Result<usize, BoxError>;
    async fn commit(&self) -> Result<(), BoxError>;
    async fn count(&self, id: &str) -> Result<usize, BoxError>;
    async fn find(&self, id: &str) -> Result<Option<ContentEntity>, BoxError>;
    async fn search(&self, params: &SearchParams) -> Result<SearchResult, BoxError>;
    async fn top_tags(&self, limit: usize) -> Result<Vec<(String, u64)>, BoxError>;
    async fn top_categories(&self, limit: usize) -> Result<Vec<(String, u64)>, BoxError>;
}
