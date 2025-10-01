use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use config::CONFIG;
use domain::model::content::{ContentEntity, FrontMatterEntity};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FrontMatterDto {
    pub title: Option<String>,
    pub date: Option<DateTime<Utc>>,
    pub description: Option<String>,
    pub draft: Option<bool>,
    pub tags: Option<Vec<String>>,
    pub categories: Option<Vec<String>>,
}

impl FrontMatterDto {
    pub fn new() -> Self {
        Self {
            title: Some(String::new()),
            date: Some(Utc::now()),
            draft: Some(true),
            tags: Some(Vec::new()),
            categories: Some(Vec::new()),
        }
    }

    pub fn default(&self) -> Self {
        Self {
            title: self.title.clone().or_else(|| Some(String::new())),
            date: self.date.or_else(|| Some(Utc::now())),
            draft: self.draft.or(Some(true)),
            tags: self.tags.clone().or_else(|| Some(Vec::new())),
            categories: self.categories.clone().or_else(|| Some(Vec::new())),
        }
    }
}

impl From<FrontMatterDto> for FrontMatterEntity {
    fn from(dto: FrontMatterDto) -> Self {
        Self {
            title: dto.title.unwrap_or_default(),
            date: dto.date.unwrap_or_else(Utc::now),
            draft: dto.draft.unwrap_or(true),
            tags: dto.tags.unwrap_or_default(),
            categories: dto.categories.unwrap_or_default(),
        }
    }
}

impl From<FrontMatterEntity> for FrontMatterDto {
    fn from(entity: FrontMatterEntity) -> Self {
        Self {
            title: Some(entity.title),
            date: Some(entity.date),
            draft: Some(entity.draft),
            tags: Some(entity.tags),
            categories: Some(entity.categories),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContentDto {
    pub id: Option<String>,
    pub matter: Option<FrontMatterDto>,
    pub body: Option<String>,
}

impl ContentDto {
    pub fn new() -> Self {
        Self {
            id: Some(Uuid::new_v4().to_string()),
            matter: Some(FrontMatterDto::new()),
            body: Some(String::new()),
        }
    }

    pub fn default(&self) -> Self {
        Self {
            id: Some(self.id.clone().unwrap_or(Uuid::new_v4().to_string())),
            matter: Some(
                self.matter
                    .as_ref()
                    .map_or_else(FrontMatterDto::new, |fm| fm.default()),
            ),
            body: self.body.clone().or_else(|| Some(String::new())),
        }
    }
}

impl From<ContentDto> for ContentEntity {
    fn from(dto: ContentDto) -> Self {
        let fm = dto.matter.unwrap_or_else(FrontMatterDto::new);

        fn normalize_list(list: Option<Vec<String>>, limit: usize, max_len: usize) -> Vec<String> {
            use std::collections::HashSet;
            let mut seen = HashSet::new();
            let mut result = Vec::new();
            if let Some(items) = list {
                for s in items {
                    let mut cleaned = s.replace('\u{3000}', " ");
                    cleaned = cleaned
                        .split_ascii_whitespace()
                        .collect::<Vec<_>>()
                        .join("");
                    cleaned = cleaned.chars().take(max_len).collect();

                    if cleaned.is_empty() {
                        continue;
                    }

                    if seen.insert(cleaned.to_string()) {
                        result.push(cleaned.to_string());
                        if result.len() >= limit {
                            break;
                        }
                    }
                }
            }
            result
        }

        fn normalize_text(text: Option<String>, max_len: usize, oneline: bool) -> String {
            let mut result = match text {
                Some(t) => t,
                None => return String::new(),
            };
            result = result.replace('\u{3000}', " ");
            if oneline {
                result = result
                    .split_ascii_whitespace()
                    .collect::<Vec<_>>()
                    .join(" ");
            } else {
                result = result.trim().to_string();
            }
            result = result.chars().take(max_len).collect();

            result
        }

        Self {
            id: dto.id.unwrap_or(Uuid::new_v4().to_string()),
            matter: FrontMatterEntity {
                title: normalize_text(fm.title, CONFIG.content.title_max_len, true),
                date: fm.date.unwrap_or_else(Utc::now),
                draft: fm.draft.unwrap_or(true),
                tags: normalize_list(fm.tags, CONFIG.content.max_tags, CONFIG.content.tag_max_len),
                categories: normalize_list(
                    fm.categories,
                    CONFIG.content.max_categories,
                    CONFIG.content.category_max_len,
                ),
            },
            body: normalize_text(dto.body, CONFIG.content.body_max_len, false),
        }
    }
}

impl From<ContentEntity> for ContentDto {
    fn from(entity: ContentEntity) -> Self {
        Self {
            id: Some(entity.id),
            matter: Some(FrontMatterDto::from(entity.matter)),
            body: Some(entity.body),
        }
    }
}
