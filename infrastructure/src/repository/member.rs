use async_trait::async_trait;
use common::types::BoxError;
use config::CONFIG;
use domain::{model::member::MemberEntity, repository::member::MemberRepository};
use std::sync::Arc;
use std::{collections::HashMap, path::PathBuf};
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct MemberRepositoryImpl {
    path: PathBuf,
    members: Arc<RwLock<HashMap<String, MemberEntity>>>,
}

impl MemberRepositoryImpl {
    pub fn new() -> Result<Self, BoxError> {
        let path = PathBuf::from(CONFIG.security.user_file.clone());
        let members = if path.exists() {
            let data = std::fs::read_to_string(&path)?;
            serde_json::from_str::<HashMap<String, MemberEntity>>(&data)?
        } else {
            HashMap::new()
        };

        Ok(Self {
            path,
            members: Arc::new(RwLock::new(members)),
        })
    }

    async fn save(&self) -> Result<(), BoxError> {
        let members = self.members.read().await;
        let data = serde_json::to_string_pretty(&*members)?;
        tokio::fs::write(&self.path, data).await?;
        Ok(())
    }
}

#[async_trait]
impl MemberRepository for MemberRepositoryImpl {
    async fn find(&self, account: &str) -> Result<Option<MemberEntity>, BoxError> {
        let members = self.members.read().await;
        Ok(members.get(account).cloned())
    }

    async fn create(&self, entity: &MemberEntity) -> Result<MemberEntity, BoxError> {
        let mut members = self.members.write().await;
        if members.contains_key(&entity.account) {
            return Err("account already exists".into());
        }
        members.insert(entity.account.clone(), entity.clone());
        Ok(entity.clone())
    }

    async fn edit(&self, entity: &MemberEntity) -> Result<Option<MemberEntity>, BoxError> {
        let mut members = self.members.write().await;
        if let Some(old) = members.get_mut(&entity.account) {
            *old = entity.clone();
            return Ok(Some(entity.clone()));
        }
        Ok(None)
    }

    async fn remove(&self, account: &str) -> Result<u32, BoxError> {
        let mut members = self.members.write().await;
        if members.remove(account).is_some() {
            Ok(1)
        } else {
            Ok(0)
        }
    }

    async fn commit(&self) -> Result<(), BoxError> {
        self.save().await
    }
}
