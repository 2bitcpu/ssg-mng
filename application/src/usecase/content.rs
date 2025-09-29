use crate::{
    errors::error::AppError,
    model::{
        content::ContentDto,
        search_engine::{SearchQueryDto, SearchRequestDto, SearchResponseDto},
    },
};
use domain::{
    Repositories,
    model::{content::ContentEntity, search_engine::SearchParams},
};
use std::sync::Arc;

pub struct ContentUseCase {
    repositories: Arc<dyn Repositories>,
}

impl ContentUseCase {
    pub fn new(repositories: Arc<dyn Repositories>) -> Self {
        Self { repositories }
    }

    pub async fn create(&self, dto: &ContentDto) -> Result<ContentDto, AppError> {
        tracing::debug!("create dto: {:?}", dto);

        let entity = ContentEntity::from(dto.clone());
        if entity.matter.title.is_empty() || entity.body.is_empty() {
            return Err(AppError::BadRequest("title or body is empty".into()));
        }

        tracing::debug!("create entity: {:?}", entity);

        let count = self.repositories.engine().count(&entity.id).await?;
        if count > 0 {
            return Err(AppError::DataConflict(entity.id).into());
        }

        let saved_entity = self.repositories.content().create(&entity).await?;

        let html_text = self
            .repositories
            .parser()
            .create(&saved_entity.clone())
            .await?;

        let index_entity = ContentEntity {
            id: saved_entity.id.clone(),
            matter: saved_entity.matter.clone(),
            body: html_text,
        };

        self.repositories.engine().create(&index_entity).await?;
        self.repositories.engine().commit().await?;

        Ok(ContentDto::from(saved_entity))
    }

    pub async fn search(&self, dto: &SearchRequestDto) -> Result<SearchResponseDto, AppError> {
        let params = SearchParams::from(dto.clone());
        let result = self.repositories.engine().search(&params).await?;
        Ok(SearchResponseDto::from(result))
    }

    pub async fn search_query(&self, dto: &SearchQueryDto) -> Result<SearchResponseDto, AppError> {
        let params = SearchParams::from(dto.clone());
        let result = self.repositories.engine().search(&params).await?;
        Ok(SearchResponseDto::from(result))
    }

    pub async fn find(&self, id: &str) -> Result<Option<ContentDto>, AppError> {
        match self.repositories.engine().find(id).await? {
            Some(plain) => {
                let content = self
                    .repositories
                    .content()
                    .find(id, &plain.matter.date)
                    .await?;
                Ok(content.map(ContentDto::from))
            }
            None => Ok(None),
        }
    }

    pub async fn remove(&self, id: &str) -> Result<serde_json::Value, AppError> {
        tracing::debug!("remove id: {}", id);

        let a = self.repositories.parser().remove(id).await?;
        let b = self.repositories.content().remove(id).await?;
        let c = self.repositories.engine().remove(id).await?;
        self.repositories.engine().commit().await?;

        tracing::debug!("remove html: {}, content: {}, index: {}", a, b, c);

        Ok(serde_json::json!({
            "id": id,
            "html": a,
            "markdown": b,
            "index": c
        }))
    }

    pub async fn edit(&self, dto: &ContentDto) -> Result<ContentDto, AppError> {
        let id = dto
            .id
            .clone()
            .ok_or_else(|| AppError::BadRequest("id is required".into()))?;

        let entity = ContentEntity::from(dto.clone());
        if entity.matter.title.is_empty() || entity.body.is_empty() {
            return Err(AppError::BadRequest("title or body is empty".into()));
        }

        let count = self.repositories.engine().count(&id).await?;
        if count == 0 {
            return Err(AppError::DataNotFound(id).into());
        }

        let a = self.repositories.parser().remove(&id).await?;
        let b = self.repositories.content().remove(&id).await?;
        let c = self.repositories.engine().remove(&id).await?;
        tracing::debug!("edit html: {}, content: {}, index: {}", a, b, c);

        let saved_entity = self.repositories.content().create(&entity).await?;

        let html_text = self
            .repositories
            .parser()
            .create(&saved_entity.clone())
            .await?;

        let index_entity = ContentEntity {
            id: saved_entity.id.clone(),
            matter: saved_entity.matter.clone(),
            body: html_text,
        };

        self.repositories.engine().edit(&index_entity).await?;
        self.repositories.engine().commit().await?;

        Ok(ContentDto::from(saved_entity))
    }

    pub async fn tags(&self, limit: usize) -> Result<Vec<(String, u64)>, AppError> {
        let limit = limit.clamp(1, 100);
        Ok(self.repositories.engine().top_tags(limit).await?)
    }

    pub async fn caregories(&self, limit: usize) -> Result<Vec<(String, u64)>, AppError> {
        let limit = limit.clamp(1, 100);
        Ok(self.repositories.engine().top_categories(limit).await?)
    }
}
