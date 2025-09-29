use crate::model::content::ContentEntity;

#[derive(Debug, Clone, PartialEq)]
pub struct SearchParams {
    pub word: Option<String>, // フリーワード タイトルと本文に対して全文検索
    pub draft: Option<bool>,  // 完全一致
    pub date_from: Option<chrono::DateTime<chrono::Utc>>, // 範囲検索(fromだけ、toだけの指定も可)
    pub date_to: Option<chrono::DateTime<chrono::Utc>>, // 範囲検索
    pub tags: Option<Vec<String>>, // 各値に完全一致
    pub categories: Option<Vec<String>>, // 各値に完全一致
    pub page: Option<usize>,  // ページ
    pub per_page: Option<usize>, // ページ内行数
}

#[derive(Debug, Clone, PartialEq)]
pub struct SearchResult {
    pub page: usize,
    pub per_page: usize,
    pub max_page: usize,
    pub overflow: bool,
    pub contents: Vec<ContentEntity>,
}
