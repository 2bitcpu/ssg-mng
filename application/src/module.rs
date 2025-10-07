use crate::usecase::{
    auth::AuthUseCase, content::ContentUseCase, image_uploader::ImageUploadUseCase,
};
use domain::Repositories;
use std::sync::Arc;

pub trait UseCaseModule: Send + Sync {
    fn content(&self) -> &ContentUseCase;
    fn auth(&self) -> &AuthUseCase;
    fn image_upload(&self) -> &ImageUploadUseCase;
}

pub struct UseCaseModuleImpl {
    content: ContentUseCase,
    auth: AuthUseCase,
    image_upload: ImageUploadUseCase,
}

impl UseCaseModuleImpl {
    pub fn new(repositories: Arc<dyn Repositories>) -> Self {
        let content = ContentUseCase::new(repositories.clone());
        let auth = AuthUseCase::new(repositories.clone());
        let image_upload = ImageUploadUseCase::new(repositories);

        Self {
            content,
            auth,
            image_upload,
        }
    }
}

#[async_trait::async_trait]
impl UseCaseModule for UseCaseModuleImpl {
    fn content(&self) -> &ContentUseCase {
        &self.content
    }

    fn auth(&self) -> &AuthUseCase {
        &self.auth
    }

    fn image_upload(&self) -> &ImageUploadUseCase {
        &self.image_upload
    }
}
