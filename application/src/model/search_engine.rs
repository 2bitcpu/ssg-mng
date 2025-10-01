use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use domain::model::{
    content::{ContentEntity, FrontMatterEntity},
    search_engine::{SearchParams, SearchResult},
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchFrontMatterDto {
    pub date: DateTime<Utc>,
    pub title: String,
    pub description: Option<String>,
    pub draft: bool,
    pub tags: Vec<String>,
    pub categories: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchContentDto {
    pub id: String,
    pub matter: SearchFrontMatterDto,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResponseDto {
    pub page: usize,
    pub per_page: usize,
    pub max_page: usize,
    pub overflow: bool,
    pub contents: Vec<SearchContentDto>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchRequestDto {
    pub word: Option<String>, // タイトルと本文に対して全文検索
    pub draft: Option<bool>,  // 完全一致
    pub date_from: Option<chrono::DateTime<chrono::Utc>>, // 範囲検索(fromだけ、toだけの指定も可)
    pub date_to: Option<chrono::DateTime<chrono::Utc>>, // 範囲検索
    pub tags: Option<Vec<String>>, // 各値に完全一致
    pub categories: Option<Vec<String>>, // 各値に完全一致
    pub page: Option<usize>,  // ページ
    pub per_page: Option<usize>, // ページ内行数
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchQueryDto {
    pub page: Option<usize>,     // ページ
    pub per_page: Option<usize>, // ページ内行数
}

impl From<SearchRequestDto> for SearchParams {
    fn from(dto: SearchRequestDto) -> Self {
        Self {
            word: dto.word,
            draft: dto.draft,
            date_from: dto.date_from,
            date_to: dto.date_to,
            tags: dto.tags,
            categories: dto.categories,
            page: dto.page,
            per_page: dto.per_page,
        }
    }
}

impl From<SearchQueryDto> for SearchParams {
    fn from(dto: SearchQueryDto) -> Self {
        Self {
            word: None,
            draft: None,
            date_from: None,
            date_to: None,
            tags: None,
            categories: None,
            page: dto.page,
            per_page: dto.per_page,
        }
    }
}

impl From<FrontMatterEntity> for SearchFrontMatterDto {
    fn from(entity: FrontMatterEntity) -> Self {
        Self {
            date: entity.date,
            title: entity.title,
            draft: entity.draft,
            tags: entity.tags,
            categories: entity.categories,
        }
    }
}

impl From<ContentEntity> for SearchContentDto {
    fn from(entity: ContentEntity) -> Self {
        let description = if entity.body.chars().count() > 100 {
            entity.body.chars().take(99).collect::<String>() + "…"
        } else {
            entity.body
        };
        Self {
            id: entity.id,
            matter: entity.matter.into(),
            description,
        }
    }
}

impl From<SearchResult> for SearchResponseDto {
    fn from(result: SearchResult) -> Self {
        Self {
            page: result.page,
            per_page: result.per_page,
            max_page: result.max_page,
            overflow: result.overflow,
            contents: result.contents.into_iter().map(|c| c.into()).collect(),
        }
    }
}
