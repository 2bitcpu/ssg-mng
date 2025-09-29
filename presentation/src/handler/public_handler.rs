use crate::errors::error::ApiError;
use application::{
    UseCaseModule,
    model::{
        content::ContentDto,
        search_engine::{SearchQueryDto, SearchRequestDto, SearchResponseDto},
    },
};
use axum::Json;
use axum::extract::{Path, Query, State};
use std::sync::Arc;

pub async fn search(
    State(usecases): State<Arc<dyn UseCaseModule>>,
    Json(dto): Json<SearchRequestDto>,
) -> Result<Json<SearchResponseDto>, ApiError> {
    let mut pub_dto = dto.clone();
    pub_dto.draft = Some(false);
    let res = usecases.content().search(&pub_dto).await?;
    Ok(Json(res))
}

pub async fn search_query(
    State(usecases): State<Arc<dyn UseCaseModule>>,
    Query(dto): Query<SearchQueryDto>,
) -> Result<Json<SearchResponseDto>, ApiError> {
    let pub_dto = SearchRequestDto {
        word: None,
        draft: Some(false),
        date_from: None,
        date_to: None,
        tags: None,
        categories: None,
        page: dto.page,
        per_page: dto.per_page,
    };
    let res = usecases.content().search(&pub_dto).await?;
    Ok(Json(res))
}

pub async fn find(
    State(usecases): State<Arc<dyn UseCaseModule>>,
    Path(id): Path<String>,
) -> Result<Json<Option<ContentDto>>, ApiError> {
    let res = usecases.content().find(&id).await?;
    let visible = res
        .as_ref()
        .filter(|content| matches!(content.matter.as_ref().and_then(|m| m.draft), Some(false)));

    Ok(Json(visible.cloned()))
}

pub async fn tags(
    State(usecases): State<Arc<dyn UseCaseModule>>,
    Path(limit): Path<usize>,
) -> Result<Json<Vec<(String, u64)>>, ApiError> {
    let res = usecases.content().tags(limit).await?;
    Ok(Json(res))
}

pub async fn caregories(
    State(usecases): State<Arc<dyn UseCaseModule>>,
    Path(limit): Path<usize>,
) -> Result<Json<Vec<(String, u64)>>, ApiError> {
    let res = usecases.content().caregories(limit).await?;
    Ok(Json(res))
}
