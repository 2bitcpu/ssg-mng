use async_trait::async_trait;

use crate::model::member::MemberEntity;
use common::types::BoxError;

#[async_trait]
pub trait MemberRepository: Send + Sync {
    async fn find(&self, account: &str) -> Result<Option<MemberEntity>, BoxError>;
    async fn create(&self, entity: &MemberEntity) -> Result<MemberEntity, BoxError>;
    async fn edit(&self, entity: &MemberEntity) -> Result<Option<MemberEntity>, BoxError>;
    async fn remove(&self, account: &str) -> Result<u32, BoxError>;
    async fn commit(&self) -> Result<(), BoxError>;
}
