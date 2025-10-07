use crate::repository::{
    content::ContentRepositoryImpl, html_parser::HtmlParserRepositoryImpl,
    image_uploader::ImageUploaderRepositoryImpl, member::MemberRepositoryImpl,
    search_engine::SearchEngineRepositoryImpl,
};
use common::types::BoxError;
use domain::{
    Repositories,
    repository::{
        content::ContentRepository, html_parser::HtmlParserRepository,
        image_uploader::ImageUploaderRepository, member::MemberRepository,
        search_engine::SearchEngineRepository,
    },
};

#[allow(dead_code)]
pub struct RepositoriesImpl {
    engine_repo: SearchEngineRepositoryImpl,
    parser_repo: HtmlParserRepositoryImpl,
    content_repo: ContentRepositoryImpl,
    member_repo: MemberRepositoryImpl,
    image_uploader_repo: ImageUploaderRepositoryImpl,
}

impl RepositoriesImpl {
    pub fn new() -> Result<Self, BoxError> {
        let engine_repo = SearchEngineRepositoryImpl::new()?;
        let parser_repo = HtmlParserRepositoryImpl::new();
        let content_repo = ContentRepositoryImpl::new();
        let member_repo = MemberRepositoryImpl::new()?;
        let image_uploader_repo = ImageUploaderRepositoryImpl::new();

        Ok(Self {
            engine_repo,
            parser_repo,
            content_repo,
            member_repo,
            image_uploader_repo,
        })
    }
}

impl Repositories for RepositoriesImpl {
    fn engine<'s>(&'s self) -> &'s dyn SearchEngineRepository {
        &self.engine_repo
    }

    fn parser<'s>(&'s self) -> &'s dyn HtmlParserRepository {
        &self.parser_repo
    }

    fn content<'s>(&'s self) -> &'s dyn ContentRepository {
        &self.content_repo
    }

    fn member<'s>(&'s self) -> &'s dyn MemberRepository {
        &self.member_repo
    }

    fn image_uploader<'s>(&'s self) -> &'s dyn ImageUploaderRepository {
        &self.image_uploader_repo
    }
}
