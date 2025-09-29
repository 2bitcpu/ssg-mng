use async_trait::async_trait;
use pulldown_cmark::{Options, Parser, html};
use std::path::PathBuf;
use tera::{Context, Tera};

use common::types::BoxError;
use config::CONFIG;
use domain::{model::content::ContentEntity, repository::html_parser::HtmlParserRepository};

#[allow(dead_code)]
pub struct HtmlParserRepositoryImpl {
    template_path: PathBuf,
    output_path: PathBuf,
}

impl HtmlParserRepositoryImpl {
    pub fn new() -> Self {
        Self {
            template_path: PathBuf::from(CONFIG.content.template_dir.clone()),
            output_path: PathBuf::from(CONFIG.content.html_dir.clone()),
        }
    }
}

#[allow(unused_variables)]
#[async_trait]
impl HtmlParserRepository for HtmlParserRepositoryImpl {
    async fn create(&self, entity: &ContentEntity) -> Result<String, BoxError> {
        let mut entity_ref = entity.clone();
        let template_path = self.template_path.clone();
        let output_path = self.output_path.clone();

        // spawn_blocking で Markdown → HTML + Tera レンダリング
        let (rendered, html_body) =
            tokio::task::spawn_blocking(move || -> Result<(String, String), BoxError> {
                // 1. Markdown → HTML
                let parser = Parser::new_ext(&entity_ref.body, Options::all());
                let mut html_body = String::new();
                html::push_html(&mut html_body, parser);

                // body を HTML に置き換え
                entity_ref.body = html_body.clone();

                // 2. Tera テンプレート読み込み
                let tera = Tera::new(&format!("{}/**/*.html", template_path.display()))?;

                // 3. Context 作成
                let mut context = Context::new();
                context.insert("content", &entity_ref);

                // 4. テンプレート適用
                let rendered = tera.render(&CONFIG.content.template_content, &context)?;

                Ok((rendered, html_body))
            })
            .await??;

        if !entity.matter.draft {
            // 5. ファイル保存先ディレクトリを作成
            let dir = self
                .output_path
                .join(entity.matter.date.format("%Y%m").to_string());
            tokio::fs::create_dir_all(&dir).await?;

            // 6. tokio::fs で HTML ファイル保存
            let output_file = dir.join(format!("{}.html", entity.id));
            tokio::fs::write(&output_file, &rendered).await?;
        }

        Ok(html_body)
    }

    async fn remove(&self, id: &str) -> Result<u64, BoxError> {
        let target_name = format!("{}.html", id);
        let mut deleted = 0u64;

        let mut stack = vec![self.output_path.clone()];

        while let Some(dir) = stack.pop() {
            let mut rd = tokio::fs::read_dir(&dir).await?;
            while let Some(entry) = rd.next_entry().await? {
                let path = entry.path();
                if path.is_dir() {
                    stack.push(path);
                } else if path.is_file() {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if name == target_name {
                            match tokio::fs::remove_file(&path).await {
                                Ok(_) => deleted += 1,
                                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                                Err(e) => return Err(Box::new(e)),
                            }
                        }
                    }
                }
            }
        }

        Ok(deleted)
    }
}
