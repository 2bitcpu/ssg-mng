use async_trait::async_trait;
use chrono::TimeZone;
use config::CONFIG;
use lindera::{mode::Mode, segmenter::Segmenter};
use lindera_tantivy::tokenizer::LinderaTokenizer;
use scraper::{ElementRef, Html, Selector};
use std::path::PathBuf;
use tantivy::{
    DocAddress, Index, IndexSettings, Order, TantivyDocument, Term,
    collector::{Count, TopDocs},
    directory::MmapDirectory,
    doc,
    query::{AllQuery, BooleanQuery, Occur, Query, QueryParser, TermQuery},
    schema::{
        FAST, Field, INDEXED, IndexRecordOption, STORED, Schema, TextFieldIndexing, TextOptions,
        Value,
    },
};

use crate::repository::index_writer_handle::{IndexWriterHandle, spawn_index_writer_task};
use common::types::BoxError;
use domain::{
    model::{
        content::{ContentEntity, FrontMatterEntity},
        search_engine::{SearchParams, SearchResult},
    },
    repository::search_engine::SearchEngineRepository,
};

#[derive(Clone)]
struct SchemaFields {
    pub id: Field,
    pub title: Field,
    pub description: Field,
    pub body: Field,
    pub draft: Field,
    pub date: Field,
    pub tags: Field,
    pub categories: Field,
}

#[allow(dead_code)]
pub struct SearchEngineRepositoryImpl {
    index: Index,
    writer_handle: IndexWriterHandle,
    fields: SchemaFields,
    search_limit: usize,
    index_limit: usize,
}

impl SearchEngineRepositoryImpl {
    pub fn new() -> Result<Self, BoxError> {
        tracing::debug!(
            "initialize search engine > index: {}, dictionary: {}, memory_budget: {}, index_limit: {}, search_limit: {}",
            CONFIG.search.index_dir.clone(),
            CONFIG.search.dictionary_dir.clone(),
            CONFIG.search.memory_budget_in_bytes / 1024,
            CONFIG.search.index_limit,
            CONFIG.search.search_limit
        );

        let index_dir = PathBuf::from(CONFIG.search.index_dir.clone());

        if !index_dir.exists() {
            std::fs::create_dir_all(&index_dir)?;
        }
        let directory = MmapDirectory::open(&index_dir)?;

        let index = if Index::exists(&directory)? {
            Index::open(directory)?
        } else {
            let schema = Self::initialize_schema()?;
            Index::create(directory, schema.clone(), IndexSettings::default())?
        };

        index.tokenizers().register(
            "lang_ja",
            Self::get_tokenizer(&CONFIG.search.dictionary_dir.clone())?,
        );
        let fields = Self::schema_to_fields(&index.schema())?;

        let writer = index.writer(CONFIG.search.memory_budget_in_bytes)?;
        let writer_handle = spawn_index_writer_task(writer);

        Ok(Self {
            index,
            writer_handle,
            fields,
            index_limit: CONFIG.search.index_limit,
            search_limit: CONFIG.search.search_limit,
        })
    }

    #[rustfmt::skip]
    fn initialize_schema() -> Result<Schema, BoxError> {
        let mut builder = Schema::builder();

        let full_match_sort = Self::get_text_field_options("raw", IndexRecordOption::Basic, true, true);
        let full_match = Self::get_text_field_options("raw", IndexRecordOption::Basic, true, false);
        let token_match = Self::get_text_field_options("lang_ja", IndexRecordOption::WithFreqsAndPositions, true, false);

        let _ = builder.add_text_field("id", full_match_sort.clone());
        let _ = builder.add_text_field("title", token_match.clone());
        let _ = builder.add_text_field("description", token_match.clone());
        let _ = builder.add_text_field("body", token_match.clone());
        let _ = builder.add_bool_field("draft", STORED | INDEXED);
        let _ = builder.add_date_field("date", STORED | INDEXED | FAST);
        let _ = builder.add_text_field("tags", full_match.clone());
        let _ = builder.add_text_field("categories", full_match.clone());
        Ok(builder.build())
    }

    fn get_text_field_options(
        tokenizer: &str,
        option: IndexRecordOption,
        stored: bool,
        fast: bool,
    ) -> TextOptions {
        let indexing = TextFieldIndexing::default()
            .set_tokenizer(tokenizer)
            .set_index_option(option);
        let mut opts = TextOptions::default().set_indexing_options(indexing);

        if stored {
            opts = opts.set_stored();
        }
        if fast {
            opts = opts.set_fast(Some(tokenizer));
        }
        opts
    }

    fn get_tokenizer(dictionary_path: &str) -> Result<LinderaTokenizer, BoxError> {
        let segmenter = Segmenter::new(
            Mode::Normal,
            lindera::dictionary::load_dictionary(dictionary_path)?,
            None,
        );
        Ok(LinderaTokenizer::from_segmenter(segmenter))
    }

    fn schema_to_fields(schema: &Schema) -> Result<SchemaFields, BoxError> {
        Ok(SchemaFields {
            id: schema.get_field("id")?,
            title: schema.get_field("title")?,
            description: schema.get_field("description")?,
            body: schema.get_field("body")?,
            draft: schema.get_field("draft")?,
            date: schema.get_field("date")?,
            tags: schema.get_field("tags")?,
            categories: schema.get_field("categories")?,
        })
    }

    fn build_query(&self, params: &SearchParams) -> Result<BooleanQuery, BoxError> {
        let mut queries: Vec<(Occur, Box<dyn Query>)> = Vec::new();

        // free word
        if let Some(word) = &params.word {
            // 全角空白があるとエラーになるので対応
            let normalize = word
                .replace('\u{3000}', " ")
                .split_ascii_whitespace()
                .collect::<Vec<_>>()
                .join(" ");
            let parser =
                QueryParser::for_index(&self.index, vec![self.fields.title, self.fields.description,self.fields.body]);
            let query = parser.parse_query(&normalize)?;
            queries.push((Occur::Must, query));
        }

        // draft
        if let Some(draft) = params.draft {
            let term = Term::from_field_bool(self.fields.draft, draft);
            let draft_query = TermQuery::new(term, IndexRecordOption::Basic);
            queries.push((Occur::Must, Box::new(draft_query)));
        }

        // date
        if params.date_from.is_some() || params.date_to.is_some() {
            // 日付範囲の取得
            let mut from = params.date_from;
            let mut to = params.date_to;

            // 範囲の正規化（逆転していたら入れ替え）
            match (from, to) {
                (Some(f), Some(t)) if f > t => {
                    std::mem::swap(&mut from, &mut to);
                }
                _ => {}
            }

            // 文字列に変換、指定なしはワイルドカード "*"
            let from_str = from
                .map(|dt| dt.to_rfc3339())
                .unwrap_or_else(|| "*".to_string());
            let to_str = to
                .map(|dt| dt.to_rfc3339())
                .unwrap_or_else(|| "*".to_string());

            // QueryParser を作成して文字列範囲検索
            let parser = QueryParser::for_index(&self.index, vec![self.fields.date]);
            let query_str = format!("date:[{} TO {}]", from_str, to_str);

            if let Ok(query) = parser.parse_query(&query_str) {
                queries.push((Occur::Must, query));
            }
        }

        // tags
        if let Some(tags) = &params.tags {
            let tag_queries: Vec<Box<dyn Query>> = tags
                .iter()
                .map(|tag| tag.trim())
                .filter(|s| !s.is_empty())
                .map(|s| {
                    // ↓ Term::from_facet から Term::from_field_text に戻します
                    let term = Term::from_field_text(self.fields.tags, s);
                    Box::new(TermQuery::new(term, IndexRecordOption::Basic)) as Box<dyn Query>
                })
                .collect();

            if !tag_queries.is_empty() {
                let mut tags_or_query = BooleanQuery::from(
                    tag_queries
                        .into_iter()
                        .map(|q| (Occur::Should, q))
                        .collect::<Vec<_>>(),
                );
                tags_or_query.set_minimum_number_should_match(1);
                queries.push((Occur::Must, Box::new(tags_or_query)));
            }
        }

        // categories
        if let Some(categories) = &params.categories {
            let category_queries: Vec<Box<dyn Query>> = categories
                .iter()
                .map(|category| category.trim())
                .filter(|s| !s.is_empty())
                .map(|s| {
                    // ↓ Term::from_facet から Term::from_field_text に戻します
                    let term = Term::from_field_text(self.fields.categories, s);
                    Box::new(TermQuery::new(term, IndexRecordOption::Basic)) as Box<dyn Query>
                })
                .collect();

            if !category_queries.is_empty() {
                let mut category_or_queries = BooleanQuery::from(
                    category_queries
                        .into_iter()
                        .map(|q| (Occur::Should, q))
                        .collect::<Vec<_>>(),
                );
                category_or_queries.set_minimum_number_should_match(1);
                queries.push((Occur::Must, Box::new(category_or_queries)));
            }
        }

        if queries.is_empty() {
            let search_all: Box<dyn Query> = Box::new(AllQuery);
            Ok(BooleanQuery::from(vec![(Occur::Must, search_all)]))
        } else {
            Ok(BooleanQuery::from(queries))
        }
    }

    async fn count_query(&self, query: BooleanQuery) -> Result<usize, BoxError> {
        let index = self.index.clone();

        tokio::task::spawn_blocking(move || -> Result<usize, BoxError> {
            let reader = index.reader()?;
            let searcher = reader.searcher();
            let count = searcher.search(&query, &Count)?;
            Ok(count)
        })
        .await?
    }

    async fn calculate_freq_map(
        &self,
        field: Field,
        limit: usize,
    ) -> Result<Vec<(String, u64)>, BoxError> {
        let index = self.index.clone();

        tokio::task::spawn_blocking(move || -> Result<Vec<(String, u64)>, BoxError> {
            let reader = index.reader()?;
            let searcher = reader.searcher();
            let mut freq_map = std::collections::HashMap::new();

            for segment_reader in searcher.segment_readers() {
                let inverted_index = segment_reader.inverted_index(field)?;
                let term_dict = inverted_index.terms();
                let mut stream = term_dict.stream()?;

                while let Some((term_bytes, term_info)) = stream.next() {
                    let term_str = std::str::from_utf8(term_bytes)?.to_string();
                    *freq_map.entry(term_str).or_insert(0) += term_info.doc_freq as u64;
                }
            }

            let mut result: Vec<(String, u64)> = freq_map.into_iter().collect();
            result.sort_by(|a, b| b.1.cmp(&a.1));
            result.truncate(limit);

            Ok(result)
        })
        .await?
    }

    async fn fetch_documents(
        &self,
        query: &BooleanQuery,
        start: usize,
        per_page: usize,
        total_count: usize,
    ) -> Result<Vec<ContentEntity>, BoxError> {
        if total_count == 0 {
            return Ok(Vec::new());
        }

        let index = self.index.clone();
        let fields = self.fields.clone();

        // Query を clone して 'static にする
        let query_cloned = query.clone();

        let results = tokio::task::spawn_blocking(move || {
            let reader = index.reader()?;
            let searcher = reader.searcher();

            let collector = TopDocs::with_limit(per_page)
                .and_offset(start)
                .order_by_fast_field("date", Order::Desc);

            let top_docs: Vec<(tantivy::DateTime, DocAddress)> =
                searcher.search(&query_cloned, &collector)?;

            let results = top_docs
                .into_iter()
                .map(|(_sort_value, doc_address)| {
                    let doc = searcher.doc(doc_address)?;
                    Ok(doc_to_entity(&doc, &fields))
                })
                .collect::<Result<Vec<_>, BoxError>>()?;

            Ok::<Vec<ContentEntity>, BoxError>(results)
        })
        .await??;

        Ok(results)
    }

    async fn register(&self, params: &ContentEntity) -> Result<ContentEntity, BoxError> {
        let mut doc = doc!(
            self.fields.id => params.id.to_string(),
            self.fields.title => params.matter.title,
            self.fields.body => params.body,
            self.fields.date => tantivy::DateTime::from_timestamp_secs(params.matter.date.timestamp()),
            self.fields.draft => params.matter.draft,
        );
        for tag in &params.matter.tags {
            doc.add_text(self.fields.tags, tag);
        }
        for category in &params.matter.categories {
            doc.add_text(self.fields.categories, category);
        }
        self.writer_handle.add_document(doc).await?;

        Ok(params.clone())
    }

    async fn check_index_limit(&self) -> Result<(), BoxError> {
        let index = self.index.clone();

        let total = tokio::task::spawn_blocking(move || -> Result<u64, BoxError> {
            let reader = index.reader()?;
            let searcher = reader.searcher();
            Ok(searcher.num_docs())
        })
        .await??;

        if total >= self.index_limit as u64 {
            return Err(format!(
                "index limit exceeded: {} (limit = {})",
                total, self.index_limit
            )
            .into());
        }

        Ok(())
    }
}

#[async_trait]
impl SearchEngineRepository for SearchEngineRepositoryImpl {
    async fn create(&self, params: &ContentEntity) -> Result<ContentEntity, BoxError> {
        tracing::debug!("create id: {}", params.id);
        self.check_index_limit().await?;

        let count = self.count(&params.id).await?;
        if count > 0 {
            return Err(format!("content with id '{}' already exists", params.id).into());
        }
        let plain_text = strip_tags(&params.body)?;
        let index_entity = ContentEntity {
            id: params.id.clone(),
            matter: params.matter.clone(),
            body: plain_text,
        };
        self.register(&index_entity).await?;
        Ok(params.clone())
    }

    async fn remove(&self, id: &str) -> Result<usize, BoxError> {
        let count = self.count(id).await?;
        if count == 0 {
            return Ok(0);
        }
        let id_term = Term::from_field_text(self.fields.id, &id);
        self.writer_handle.delete_term(id_term).await?;
        Ok(count)
    }

    async fn edit(&self, params: &ContentEntity) -> Result<Option<ContentEntity>, BoxError> {
        tracing::debug!("edit id: {}", params.id);
        if self.find(&params.id).await?.is_none() {
            return Ok(None);
        }
        let removed_count = self.remove(&params.id).await?;
        if removed_count == 0 {
            return Ok(None);
        }
        let plain_text = strip_tags(&params.body)?;
        let index_entity = ContentEntity {
            id: params.id.clone(),
            matter: params.matter.clone(),
            body: plain_text,
        };
        self.register(&index_entity).await?;
        Ok(Some(params.clone()))
    }

    async fn commit(&self) -> Result<(), BoxError> {
        self.writer_handle.commit().await
    }

    async fn find(&self, id: &str) -> Result<Option<ContentEntity>, BoxError> {
        let id = id.to_string();
        let fields = self.fields.clone();
        let index = self.index.clone();

        tokio::task::spawn_blocking(move || -> Result<Option<ContentEntity>, BoxError> {
            let reader = index.reader()?;
            let searcher = reader.searcher();

            let term = Term::from_field_text(fields.id, &id);
            let query = TermQuery::new(term, IndexRecordOption::Basic);

            let top_docs = searcher.search(&query, &TopDocs::with_limit(2))?;

            if top_docs.is_empty() {
                return Ok(None);
            }

            if top_docs.len() > 1 {
                return Err(format!(
                    "More than one document found for unique ID, index may be corrupted for id: {}",
                    id
                )
                .into());
            }

            let doc_address = top_docs[0].1;
            let doc = searcher.doc(doc_address)?;
            Ok(Some(doc_to_entity(&doc, &fields)))
        })
        .await?
    }

    async fn count(&self, id: &str) -> Result<usize, BoxError> {
        let index = self.index.clone();
        let fields = self.fields.clone();
        let id = id.to_string();

        let count = tokio::task::spawn_blocking({
            let id = id.clone();
            move || -> Result<usize, BoxError> {
                let id_term = Term::from_field_text(fields.id, &id);
                let reader = index.reader()?;
                let searcher = reader.searcher();
                let query = TermQuery::new(id_term.clone(), IndexRecordOption::Basic);
                let count = searcher.search(&query, &Count)?;
                Ok(count)
            }
        })
        .await??;

        Ok(count)
    }

    async fn search(&self, params: &SearchParams) -> Result<SearchResult, BoxError> {
        tracing::debug!("search params: {:#?}", params);

        let query = self.build_query(&params)?;

        let mut total_count = self.count_query(query.clone()).await?;

        tracing::debug!("total count: {}", total_count);

        let overflow = if total_count > self.search_limit {
            total_count = self.search_limit;
            true
        } else {
            false
        };

        let (start, _end, page, per_page, max_page) =
            paginate(total_count, params.page, params.per_page);

        let contents = self
            .fetch_documents(&query, start, per_page as usize, total_count as usize)
            .await?;

        // tracing::debug!("contents: {:#?}", contents);

        Ok(SearchResult {
            page,
            per_page,
            max_page,
            overflow,
            contents,
        })
    }

    async fn top_tags(&self, limit: usize) -> Result<Vec<(String, u64)>, BoxError> {
        Ok(self.calculate_freq_map(self.fields.tags, limit).await?)
    }

    async fn top_categories(&self, limit: usize) -> Result<Vec<(String, u64)>, BoxError> {
        Ok(self
            .calculate_freq_map(self.fields.categories, limit)
            .await?)
    }
}

// helper
#[allow(dead_code)]
fn doc_to_entity(doc: &TantivyDocument, fields: &SchemaFields) -> ContentEntity {
    ContentEntity {
        id: get_str(doc, fields.id),
        matter: FrontMatterEntity {
            title: get_str(doc, fields.title),
            description: get_str(doc, fields.description),
            date: get_datetime(doc, fields.date),
            draft: get_bool(doc, fields.draft),
            tags: get_str_list(doc, fields.tags),
            categories: get_str_list(doc, fields.categories),
        },
        body: get_str(doc, fields.body),
    }
}

fn get_str(doc: &TantivyDocument, field: Field) -> String {
    doc.get_first(field)
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_default()
}

fn get_str_list(doc: &TantivyDocument, field: Field) -> Vec<String> {
    doc.get_all(field)
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect()
}

fn get_bool(doc: &TantivyDocument, field: Field) -> bool {
    doc.get_first(field)
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}

fn get_datetime(doc: &TantivyDocument, field: Field) -> chrono::DateTime<chrono::Utc> {
    let ts = doc
        .get_first(field)
        .and_then(|v| v.as_datetime())
        .unwrap_or_default()
        .into_timestamp_secs();
    chrono::Utc.timestamp_opt(ts, 0).unwrap()
}

fn paginate(
    total: usize,
    page: Option<usize>,
    per_page: Option<usize>,
) -> (usize, usize, usize, usize, usize) {
    tracing::debug!(
        "paginate total: {}, page: {:?}, per_page: {:?}",
        total,
        page,
        per_page
    );
    let per_page = per_page.unwrap_or(10).max(1);
    let mut page = page.unwrap_or(1).max(1);

    let pages = if total == 0 {
        0
    } else {
        (total + per_page - 1) / per_page
    };

    if pages > 0 && page > pages {
        page = pages;
    }

    let start = ((page - 1) * per_page).min(total) as usize;
    let end = ((start + per_page).min(total)) as usize;

    tracing::debug!(
        "paginate exit: {}, end: {}, page: {}, per_page: {}, pages: {}",
        start,
        end,
        page,
        per_page,
        pages
    );
    (start, end, page, per_page, pages)
}

fn recursive_strip_tags(element: &ElementRef, plain_text: &mut String) {
    let tag_name = element.value().name();

    if tag_name == "script" || tag_name == "style" || tag_name == "meta" {
        return;
    }

    if tag_name == "img" {
        if let Some(alt) = element.value().attr("alt") {
            plain_text.push_str(alt);
            plain_text.push(' ');
        }
        return;
    }

    for child in element.children() {
        if let Some(text) = child.value().as_text() {
            plain_text.push_str(text);
            plain_text.push(' ');
        } else if let Some(ce) = ElementRef::wrap(child.clone()) {
            recursive_strip_tags(&ce, plain_text);
        }
    }
}

fn strip_tags(html: &str) -> Result<String, BoxError> {
    let document = Html::parse_document(html);
    let se = Selector::parse("body").map_err(|_| "Failed to parse selector")?;
    let root = document
        .select(&se)
        .next()
        .unwrap_or_else(|| document.root_element());

    let mut plain_text = String::new();
    recursive_strip_tags(&root, &mut plain_text);

    Ok(plain_text
        .split_ascii_whitespace()
        .collect::<Vec<_>>()
        .join(" "))
}
