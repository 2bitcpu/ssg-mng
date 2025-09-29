use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FrontMatterEntity {
    pub date: DateTime<Utc>,
    pub title: String,
    pub draft: bool,
    pub tags: Vec<String>,
    pub categories: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContentEntity {
    pub id: String,
    pub matter: FrontMatterEntity,
    pub body: String,
}
