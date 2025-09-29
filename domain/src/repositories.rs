use crate::repository::content::ContentRepository;
use crate::repository::html_parser::HtmlParserRepository;
use crate::repository::member::MemberRepository;
use crate::repository::search_engine::SearchEngineRepository;

pub trait Repositories: Send + Sync {
    fn engine<'s>(&'s self) -> &'s dyn SearchEngineRepository;
    fn parser<'s>(&'s self) -> &'s dyn HtmlParserRepository;
    fn content<'s>(&'s self) -> &'s dyn ContentRepository;
    fn member<'s>(&'s self) -> &'s dyn MemberRepository;
}
