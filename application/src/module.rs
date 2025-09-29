use crate::usecase::{auth::AuthUseCase, content::ContentUseCase};
use domain::Repositories;
use std::sync::Arc;

pub trait UseCaseModule: Send + Sync {
    fn content(&self) -> &ContentUseCase;
    fn auth(&self) -> &AuthUseCase;
}

pub struct UseCaseModuleImpl {
    content: ContentUseCase,
    auth: AuthUseCase,
}

impl UseCaseModuleImpl {
    pub fn new(repositories: Arc<dyn Repositories>) -> Self {
        let content = ContentUseCase::new(repositories.clone());
        let auth = AuthUseCase::new(repositories);

        Self { content, auth }
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
}
