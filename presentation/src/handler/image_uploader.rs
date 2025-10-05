use crate::errors::error::ApiError;
use crate::middleware::auth::AuthMember;
use application::AppError;

use application::{
    UseCaseModule,
    model::image_uploader::{ImageUploadDto, ImageUploadResponseDto},
};
use axum::{
    Extension, Json,
    extract::{Multipart, State},
};
use std::sync::Arc;

pub async fn upload(
    State(usecases): State<Arc<dyn UseCaseModule>>,
    Extension(_member): Extension<AuthMember>,
    multipart: Multipart,
) -> Result<Json<ImageUploadResponseDto>, ApiError> {
    tracing::debug!("start handler upload");

    let dto = ImageUploadDto::new(extract_file_bytes(multipart, "image").await?);

    tracing::debug!("step(2) handler upload");

    let res = usecases.image_upload().upload(&dto).await?;

    tracing::debug!("step(3) handler upload");

    Ok(Json(res))
}

async fn extract_file_bytes(
    mut multipart: Multipart,
    field_name: &str,
) -> Result<Vec<u8>, AppError> {
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::Unexpected(e.into()))?
    {
        if let Some(name) = field.name() {
            if name == field_name {
                let bytes = field
                    .bytes()
                    .await
                    .map_err(|e| AppError::Unexpected(e.into()))?;
                return Ok(bytes.to_vec());
            }
        }
    }

    Err(AppError::BadRequest(format!(
        "field '{}' not found in multipart data",
        field_name
    )))
}
