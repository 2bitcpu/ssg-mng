use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use config::CONFIG;
use domain::model::{
    content::ContentEntity,
    search_engine::{SearchParams, SearchResult},
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchFrontMatterDto {
    pub date: DateTime<Utc>,
    pub title: String,
    pub description: String,
    pub draft: bool,
    pub tags: Vec<String>,
    pub categories: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchContentDto {
    pub id: String,
    pub matter: SearchFrontMatterDto,
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

impl From<ContentEntity> for SearchContentDto {
    fn from(entity: ContentEntity) -> Self {
        let description = entity
            .matter
            .description
            .as_ref()
            .map(|d| d.trim())
            .filter(|d| !d.is_empty())
            .map(|d| d.to_string())
            .unwrap_or_else(|| {
                let body = &entity.body;
                if body.chars().count() > CONFIG.content.description_max_len {
                    body.chars()
                        .take(CONFIG.content.description_max_len - 1)
                        .collect::<String>()
                        + "…"
                } else {
                    body.clone()
                }
            });

        Self {
            id: entity.id,
            matter: SearchFrontMatterDto {
                date: entity.matter.date,
                title: entity.matter.title,
                description,
                draft: entity.matter.draft,
                tags: entity.matter.tags,
                categories: entity.matter.categories,
            },
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
