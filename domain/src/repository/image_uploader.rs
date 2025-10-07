use async_trait::async_trait;
use common::types::BoxError;

#[async_trait]
pub trait ImageUploaderRepository: Send + Sync {
    async fn save(&self, data: &[u8]) -> Result<(String, String), BoxError>;
}
