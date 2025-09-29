use crate::errors::error::ApiError;
use crate::middleware::auth::AuthMember;

use application::{
    UseCaseModule,
    model::{
        content::ContentDto,
        search_engine::{SearchQueryDto, SearchRequestDto, SearchResponseDto},
    },
};
use axum::{
    Extension, Json,
    extract::{Path, Query, State},
};
use std::sync::Arc;

pub async fn create(
    State(usecases): State<Arc<dyn UseCaseModule>>,
    Extension(_member): Extension<AuthMember>,
    Json(dto): Json<ContentDto>,
) -> Result<Json<ContentDto>, ApiError> {
    let res = usecases.content().create(&dto).await?;
    Ok(Json(res))
}

pub async fn search(
    State(usecases): State<Arc<dyn UseCaseModule>>,
    Extension(_member): Extension<AuthMember>,
    Json(dto): Json<SearchRequestDto>,
) -> Result<Json<SearchResponseDto>, ApiError> {
    let res = usecases.content().search(&dto).await?;
    Ok(Json(res))
}

pub async fn search_query(
    State(usecases): State<Arc<dyn UseCaseModule>>,
    Extension(_member): Extension<AuthMember>,
    Query(dto): Query<SearchQueryDto>,
) -> Result<Json<SearchResponseDto>, ApiError> {
    let res = usecases.content().search_query(&dto).await?;
    Ok(Json(res))
}

pub async fn find(
    State(usecases): State<Arc<dyn UseCaseModule>>,
    Extension(_member): Extension<AuthMember>,
    Path(id): Path<String>,
) -> Result<Json<Option<ContentDto>>, ApiError> {
    let res = usecases.content().find(&id).await?;
    Ok(Json(res))
}

pub async fn remove(
    State(usecases): State<Arc<dyn UseCaseModule>>,
    Extension(_member): Extension<AuthMember>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let res = usecases.content().remove(&id).await?;
    Ok(Json(res))
}

pub async fn edit(
    State(usecases): State<Arc<dyn UseCaseModule>>,
    Extension(_member): Extension<AuthMember>,
    Json(dto): Json<ContentDto>,
) -> Result<Json<ContentDto>, ApiError> {
    let res = usecases.content().edit(&dto).await?;
    Ok(Json(res))
}

pub async fn tags(
    State(usecases): State<Arc<dyn UseCaseModule>>,
    Extension(_member): Extension<AuthMember>,
    Path(limit): Path<usize>,
) -> Result<Json<Vec<(String, u64)>>, ApiError> {
    let res = usecases.content().tags(limit).await?;
    Ok(Json(res))
}

pub async fn caregories(
    State(usecases): State<Arc<dyn UseCaseModule>>,
    Extension(_member): Extension<AuthMember>,
    Path(limit): Path<usize>,
) -> Result<Json<Vec<(String, u64)>>, ApiError> {
    let res = usecases.content().caregories(limit).await?;
    Ok(Json(res))
}
