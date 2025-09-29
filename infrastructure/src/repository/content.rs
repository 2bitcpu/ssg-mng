use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::path::PathBuf;

use common::types::BoxError;
use config::CONFIG;
use domain::{
    model::content::{ContentEntity, FrontMatterEntity},
    repository::content::ContentRepository,
};

#[allow(dead_code)]
pub struct ContentRepositoryImpl {
    output_path: PathBuf,
}

impl ContentRepositoryImpl {
    pub fn new() -> Self {
        Self {
            output_path: PathBuf::from(CONFIG.content.markdown_dir.clone()),
        }
    }
}

#[allow(unused_variables)]
#[async_trait]
impl ContentRepository for ContentRepositoryImpl {
    async fn create(&self, entity: &ContentEntity) -> Result<ContentEntity, BoxError> {
        let dir = self
            .output_path
            .join(entity.matter.date.format("%Y%m").to_string());
        tokio::fs::create_dir_all(&dir).await?;

        let front_matter_yaml = serde_yaml::to_string(&entity.matter)?;

        let file_content = format!("---\n{}\n---\n{}", front_matter_yaml.trim(), entity.body);

        let file_path = dir.join(format!("{}.md", entity.id.clone()));
        tokio::fs::write(&file_path, file_content).await?;

        Ok(ContentEntity {
            id: entity.id.clone(),
            matter: entity.matter.clone(),
            body: entity.body.clone(),
        })
    }

    async fn find(
        &self,
        id: &str,
        date: &DateTime<Utc>,
    ) -> Result<Option<ContentEntity>, BoxError> {
        let file_path = self
            .output_path
            .join(date.format("%Y%m").to_string())
            .join(format!("{}.md", id));

        if !file_path.exists() {
            return Ok(None);
        }

        let content = tokio::fs::read_to_string(&file_path).await?;

        if !content.starts_with("---\n") {
            return Err(BoxError::from(format!(
                "Front matter missing in file: {}",
                file_path.display()
            )));
        }

        let fm_end = content.find("\n---\n").ok_or_else(|| {
            BoxError::from(format!(
                "Front matter not closed properly in file: {}",
                file_path.display()
            ))
        })?;

        let matter_yaml = &content[4..fm_end];
        let body = &content[fm_end + 5..];

        let matter: FrontMatterEntity = serde_yaml::from_str(matter_yaml)?;

        Ok(Some(ContentEntity {
            id: id.to_string(),
            matter,
            body: body.to_string(),
        }))
    }

    async fn remove(&self, id: &str) -> Result<u64, BoxError> {
        let target_name = format!("{}.html", id);
        let mut deleted = 0u64;

        let mut stack = vec![self.output_path.clone()];

        // 探索して全て削除する
        while let Some(dir) = stack.pop() {
            let mut rd = tokio::fs::read_dir(&dir).await?;
            while let Some(entry) = rd.next_entry().await? {
                let path = entry.path();
                if path.is_dir() {
                    stack.push(path);
                } else if path.is_file() {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if name == target_name {
                            match tokio::fs::remove_file(&path).await {
                                Ok(_) => deleted += 1,
                                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                                Err(e) => return Err(Box::new(e)),
                            }
                        }
                    }
                }
            }
        }

        Ok(deleted)
    }
}
