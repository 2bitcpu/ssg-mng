use crate::errors::error::AppError;
use crate::model::image_uploader::{ImageUploadDto, ImageUploadResponseDto};
use domain::Repositories;
use std::sync::Arc;

pub struct ImageUploadUseCase {
    repositories: Arc<dyn Repositories>,
}

impl ImageUploadUseCase {
    pub fn new(repositories: Arc<dyn Repositories>) -> Self {
        Self { repositories }
    }

    pub async fn upload(&self, dto: &ImageUploadDto) -> Result<ImageUploadResponseDto, AppError> {
        tracing::debug!("start usecase upload");

        let (default, thumb) = self
            .repositories
            .image_uploader()
            .save(&dto.data)
            .await
            .map_err(|e| AppError::from(e))?;

        Ok(ImageUploadResponseDto { default, thumb })
    }
}
