#!/bin/bash

if [[ "$1" == "-d" || "$1" == "--del" ]]; then
    find . -mindepth 1 -maxdepth 1 ! \( -name "*.md" -o -name "*.sh" -o -name "*.yaml" \) -exec rm -rf {} +
elif find . -mindepth 1 -maxdepth 1 ! \( -name "*.md" -o -name "*.sh" -o -name "*.yaml" \) | grep -q .; then
    echo "Project already exists."
    exit 1
fi

cargo new _tmp
cd _tmp
cargo add tokio --features macros,rt-multi-thread,signal --no-default-features
cargo add serde --features derive --no-default-features
cargo add serde_json --features std --no-default-features
cargo add serde_yaml --no-default-features
cargo add chrono --features serde,now --no-default-features
cargo add async-trait --no-default-features
cargo add axum --features macros
cargo add axum-extra --features typed-header --no-default-features
cargo add tower --features timeout --no-default-features
cargo add tower-http --features fs,cors --no-default-features
cargo add uuid --features v4,serde --no-default-features
cargo add jsonwebtoken --no-default-features
cargo add argon2 --features alloc,password-hash,std --no-default-features
cargo add password-hash --features getrandom --no-default-features
cargo add clap --features derive 
cargo add once_cell --features std --no-default-features
cargo add tracing --no-default-features
cargo add tracing-subscriber --no-default-features --features fmt,env-filter

cargo add lindera
cargo add tantivy
cargo add regex
cargo add tera --features chrono --no-default-features
cargo add scraper
cargo add tempfile

cd ..

cat <<EOF > Cargo.toml
[workspace.package]
version = "0.1.0"
edition = "2024"

[workspace]
resolver = "2"
members = []

[workspace.dependencies]
lindera-tantivy = { git = "https://github.com/lindera/lindera-tantivy" }
EOF

sed '1,/\[dependencies\]/d' _tmp/Cargo.toml >> Cargo.toml
rm -rf _tmp

cargo new config --lib
cargo new common --lib
cargo new domain --lib
cargo new infrastructure --lib
cargo new application --lib
cargo new presentation --lib
cargo new ssg-mhg

find . \( -name ".git" -o -name ".gitignore" \) -exec rm -rf {} +

cat <<EOF >> .gitignore
target
Cargo.lock
.DS_Store
EOF

cat <<EOF >> Cargo.toml

config = { path = "config" }
common = { path = "common" }
domain = { path = "domain" }
infrastructure = { path = "infrastructure" }
application = { path = "application" }
presentation = { path = "presentation" }
# ssg-mhg = { path = "ssg-mhg" }

[profile.release]
opt-level = "z"
debug = false
lto = true
strip = true
codegen-units = 1
panic = "abort"
EOF

##############################
# common
##############################
cat <<EOF > common/src/lib.rs
pub mod types;
EOF

cat <<EOF > common/src/types.rs
pub type BoxError = Box<dyn std::error::Error + Send + Sync>;
EOF

##############################
# config
##############################
cat <<EOF >> config/Cargo.toml
clap.workspace = true
serde.workspace = true
serde_yaml.workspace = true
tracing-subscriber.workspace = true
uuid.workspace = true
once_cell.workspace = true
EOF

cat <<EOF > config/src/lib.rs
mod config;
pub use config::CONFIG;
EOF

cat <<EOF > config/src/config.rs
use clap::Parser;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tracing_subscriber::EnvFilter;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub content: ContentConfig,
    pub search: SearchConfig,
    pub security: SecurityConfig,
    pub log: LogConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,      // default "0.0.0.0:3000"
    pub cors: Vec<String>, // default  []
    #[serde(rename = "static")]
    pub static_dir: Option<String>, // default None
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ContentConfig {
    pub markdown_dir: String,     // default "output/markdown"
    pub html_dir: String,         // default "output/public_html"
    pub title_max_len: u32,       // default 80 (80〜160) clamp
    pub body_max_len: u32,        // default 3000 (100〜5000) clamp
    pub max_tags: u32,            // default 6 (1〜100) clamp
    pub max_categories: u32,      // default 3 (1〜5) clamp
    pub template_dir: String,     // default "templates"
    pub template_content: String, // default "content.html"
    pub template_index: String,   // default "index.html"
    pub template_list: String,    // default "list.html"
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchConfig {
    pub dictionary_dir: String, // default "dictionary"
    pub index_dir: String,      // default "output/.index"
    pub max_index_count: u32,   // default 3000 (100〜10000) clamp
    pub search_limit: u32,      // default 1000 (100〜10000) clamp
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SecurityConfig {
    pub issuer: String,    // default exe basename
    pub secret: String,    // default uuid v4
    pub expire: i64,       // seconds; default 86400; clamp 180..=90days
    pub user_file: String, // default "config/user.dat"
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LogConfig {
    pub level: Option<String>, // default None
}

impl Default for Config {
    fn default() -> Self {
        let exe_name = Config::exe_basename();
        Self {
            server: ServerConfig {
                host: "0.0.0.0:3000".to_string(),
                cors: vec![],
                static_dir: None,
            },
            content: ContentConfig {
                markdown_dir: "output/markdown".to_string(),
                html_dir: "output/public_html".to_string(),
                title_max_len: 80,
                body_max_len: 3000,
                max_tags: 6,
                max_categories: 3,
                template_dir: "templates".to_string(),
                template_content: "content.html".to_string(),
                template_index: "index.html".to_string(),
                template_list: "list.html".to_string(),
            },
            search: SearchConfig {
                dictionary_dir: "dictionary".to_string(),
                index_dir: "output/.index".to_string(),
                max_index_count: 3000,
                search_limit: 1000,
            },
            security: SecurityConfig {
                issuer: exe_name.clone(),
                secret: Uuid::new_v4().to_string(),
                expire: 60 * 60 * 24, // 1 day
                user_file: "config/user.dat".to_string(),
            },
            log: LogConfig { level: None },
        }
    }
}

pub static CONFIG: Lazy<Config> = Lazy::new(|| {
    let mut cfg = Config::default();
    let exe_name = Config::exe_basename();
    let filename = format!("{exe_name}.config.yaml");

    let paths = vec![
        format!("/etc/{exe_name}/{filename}"),
        format!(
            "{}/{}",
            std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(|x| x.display().to_string()))
                .unwrap_or_else(|| ".".to_string()),
            filename
        ),
        filename.clone(),
    ];

    for path in paths {
        if Path::new(&path).exists() {
            match fs::read_to_string(&path) {
                Ok(content) => match serde_yaml::from_str::<PartialConfig>(&content) {
                    Ok(partial) => cfg.merge(partial),
                    Err(e) => eprintln!("Failed to parse config '{}': {}", path, e),
                },
                Err(e) => eprintln!("Failed to read config '{}': {}", path, e),
            }
        }
    }

    cfg.validate_and_normalize();

    let cli = Cli::parse();
    cfg.apply_cli(&cli);

    cfg
});

#[derive(Debug, Deserialize)]
struct PartialConfig {
    server: Option<PartialServerConfig>,
    content: Option<PartialContentConfig>,
    search: Option<PartialSearchConfig>,
    security: Option<PartialSecurityConfig>,
    log: Option<PartialLogConfig>,
}

#[derive(Debug, Deserialize)]
struct PartialServerConfig {
    host: Option<String>,
    cors: Option<Vec<String>>,
    #[serde(rename = "static")]
    static_dir: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PartialContentConfig {
    markdown_dir: Option<String>,
    html_dir: Option<String>,
    title_max_len: Option<u32>,
    body_max_len: Option<u32>,
    max_tags: Option<u32>,
    max_categories: Option<u32>,
    template_dir: Option<String>,
    template_content: Option<String>,
    template_index: Option<String>,
    template_list: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PartialSearchConfig {
    dictionary_dir: Option<String>,
    index_dir: Option<String>,
    max_index_count: Option<u32>,
    search_limit: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct PartialSecurityConfig {
    issuer: Option<String>,
    secret: Option<String>,
    expire: Option<i64>,
    user_file: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PartialLogConfig {
    level: Option<String>,
}

impl Config {
    #[rustfmt::skip]
    fn merge(&mut self, p: PartialConfig) {
        if let Some(server) = p.server {
            if let Some(h) = server.host { self.server.host = h; }
            if let Some(c) = server.cors { self.server.cors = c; }
            if let Some(s) = server.static_dir { self.server.static_dir = Some(s); }
        }
        if let Some(content) = p.content {
            if let Some(v) = content.markdown_dir { self.content.markdown_dir = v; }
            if let Some(v) = content.html_dir { self.content.html_dir = v; }
            if let Some(v) = content.title_max_len { self.content.title_max_len = v; }
            if let Some(v) = content.body_max_len { self.content.body_max_len = v; }
            if let Some(v) = content.max_tags { self.content.max_tags = v; }
            if let Some(v) = content.max_categories { self.content.max_categories = v; }
            if let Some(v) = content.template_dir { self.content.template_dir = v; }
            if let Some(v) = content.template_content { self.content.template_content = v; }
            if let Some(v) = content.template_index { self.content.template_index = v; }
            if let Some(v) = content.template_list { self.content.template_list = v; }
        }
        if let Some(search) = p.search {
            if let Some(v) = search.dictionary_dir { self.search.dictionary_dir = v; }
            if let Some(v) = search.index_dir { self.search.index_dir = v; }
            if let Some(v) = search.max_index_count { self.search.max_index_count = v; }
            if let Some(v) = search.search_limit { self.search.search_limit = v; }
        }
        if let Some(security) = p.security {
            if let Some(v) = security.issuer { self.security.issuer = v; }
            if let Some(v) = security.secret { self.security.secret = v; }
            if let Some(v) = security.expire { self.security.expire = v; }
            if let Some(v) = security.user_file { self.security.user_file = v; }
        }
        if let Some(log) = p.log {
            if let Some(v) = log.level { self.log.level = Some(v); }
        }
    }

    fn validate_and_normalize(&mut self) {
        if let Some(ref level) = self.log.level {
            if EnvFilter::try_new(level).is_err() {
                eprintln!(
                    "Invalid log filter expression: '{}'. Logging will be disabled.",
                    level
                );
                self.log.level = None;
            }
        }

        if let Some(ref dir) = self.server.static_dir {
            if !Path::new(dir).is_dir() {
                eprintln!(
                    "Static directory '{}' does not exist. Static serving will be disabled.",
                    dir
                );
                self.server.static_dir = None;
            }
        }

        for (name, path) in &[
            ("markdown_dir", &self.content.markdown_dir),
            ("html_dir", &self.content.html_dir),
            ("template_dir", &self.content.template_dir),
        ] {
            if !Path::new(path).is_dir() {
                panic!("Configured {} '{}' does not exist.", name, path);
            }
        }

        for (name, path) in &[
            ("dictionary_dir", &self.search.dictionary_dir),
            ("index_dir", &self.search.index_dir),
        ] {
            if !Path::new(path).is_dir() {
                panic!("Configured {} '{}' does not exist.", name, path);
            }
        }

        // Content
        let (t_old, t_new) = (
            self.content.title_max_len,
            clamp_u32(self.content.title_max_len, 80, 160),
        );
        if t_old != t_new {
            eprintln!(
                "title_max_len {} is out of range [80,160], rounded to {}.",
                t_old, t_new
            );
            self.content.title_max_len = t_new;
        }
        let (b_old, b_new) = (
            self.content.body_max_len,
            clamp_u32(self.content.body_max_len, 100, 5000),
        );
        if b_old != b_new {
            eprintln!(
                "body_max_len {} is out of range [100,5000], rounded to {}.",
                b_old, b_new
            );
            self.content.body_max_len = b_new;
        }
        let (tag_old, tag_new) = (
            self.content.max_tags,
            clamp_u32(self.content.max_tags, 1, 100),
        );
        if tag_old != tag_new {
            eprintln!(
                "max_tags {} is out of range [1,100], rounded to {}.",
                tag_old, tag_new
            );
            self.content.max_tags = tag_new;
        }
        let (cat_old, cat_new) = (
            self.content.max_categories,
            clamp_u32(self.content.max_categories, 1, 5),
        );
        if cat_old != cat_new {
            eprintln!(
                "max_categories {} is out of range [1,5], rounded to {}.",
                cat_old, cat_new
            );
            self.content.max_categories = cat_new;
        }

        let (mi_old, mi_new) = (
            self.search.max_index_count,
            clamp_u32(self.search.max_index_count, 100, 10_000),
        );
        if mi_old != mi_new {
            eprintln!(
                "max_index_count {} out of range [100,10000], rounded to {}.",
                mi_old, mi_new
            );
            self.search.max_index_count = mi_new;
        }
        let (sl_old, sl_new) = (
            self.search.search_limit,
            clamp_u32(self.search.search_limit, 100, 10_000),
        );
        if sl_old != sl_new {
            eprintln!(
                "search_limit {} out of range [100,10000], rounded to {}.",
                sl_old, sl_new
            );
            self.search.search_limit = sl_new;
        }

        // Security: expire clamp (180 ..= 90 days)
        let min_expire: i64 = 180;
        let max_expire: i64 = 60 * 60 * 24 * 90;
        if self.security.expire < min_expire {
            eprintln!(
                "security.expire {} is less than {}, rounded up.",
                self.security.expire, min_expire
            );
            self.security.expire = min_expire;
        } else if self.security.expire > max_expire {
            eprintln!(
                "security.expire {} is greater than {}, rounded down.",
                self.security.expire, max_expire
            );
            self.security.expire = max_expire;
        }
    }

    /// Overwide from CLI
    fn apply_cli(&mut self, cli: &Cli) {
        // server
        if let Some(host) = &cli.host {
            self.server.host = host.clone();
        }
        if cli.no_cors {
            self.server.cors.clear();
        } else if let Some(cors) = &cli.cors {
            self.server.cors = cors.clone();
        }
        if cli.no_static {
            self.server.static_dir = None;
        } else if let Some(dir) = &cli.static_dir {
            if let Some(s) = dir.to_str() {
                self.server.static_dir = Some(s.to_string());
                if !Path::new(s).is_dir() {
                    eprintln!(
                        "CLI: static directory '{}' does not exist. Static serving will be disabled.",
                        s
                    );
                    self.server.static_dir = None;
                }
            } else {
                eprintln!("CLI: Invalid static directory path.");
            }
        }

        // log
        if cli.no_log {
            self.log.level = None;
        } else if let Some(level) = &cli.log_level {
            if EnvFilter::try_new(level).is_ok() {
                self.log.level = Some(level.clone());
            } else {
                eprintln!("CLI: invalid log level '{}', ignoring.", level);
            }
        }
    }

    fn exe_basename() -> String {
        std::env::current_exe()
            .ok()
            .and_then(|path| path.file_stem().map(|s| s.to_string_lossy().to_string()))
            .unwrap_or_else(|| "unknown".to_string())
    }
}

fn clamp_u32(v: u32, lo: u32, hi: u32) -> u32 {
    if v < lo {
        lo
    } else if v > hi {
        hi
    } else {
        v
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Cli {
    #[arg(long)]
    pub host: Option<String>,

    #[arg(long)]
    pub cors: Option<Vec<String>>,
    #[arg(long)]
    pub no_cors: bool,

    #[arg(long)]
    pub static_dir: Option<PathBuf>,
    #[arg(long)]
    pub no_static: bool,

    #[arg(long)]
    pub log_level: Option<String>,
    #[arg(long)]
    pub no_log: bool,
}
EOF

##############################
# domain
##############################
cat <<EOF >> domain/Cargo.toml
serde.workspace = true
chrono.workspace = true
async-trait.workspace = true

common.workspace = true
EOF

mkdir -p domain/src/model

cat <<EOF > domain/src/model/content.rs
EOF

cat <<EOF > domain/src/model/search_engine.rs
EOF

cat <<EOF > domain/src/model/mod.rs
pub mod content;
pub mod search_engine;
EOF

mkdir -p domain/src/repository

cat <<EOF > domain/src/repository/content.rs
EOF

cat <<EOF > domain/src/repository/search_engine.rs
EOF

cat <<EOF > domain/src/repository/mod.rs
pub mod content;
pub mod search_engine;
EOF

cat <<EOF > domain/src/repositories.rs
EOF

cat <<EOF > domain/src/lib.rs
pub mod model;
pub mod repository;

mod repositories;
pub use repositories::Repositories;
EOF

##############################
# infrastructure
##############################
cat <<EOF >> infrastructure/Cargo.toml
serde.workspace = true
chrono.workspace = true
async-trait.workspace = true

common.workspace = true
domain.workspace = true
EOF

mkdir -p infrastructure/src/repository

cat <<EOF > infrastructure/src/repository/content.rs
EOF

cat <<EOF > infrastructure/src/repository/search_engine.rs
EOF

cat <<EOF > infrastructure/src/repository/mod.rs
pub mod content;
pub mod search_engine;
EOF

cat <<EOF > infrastructure/src/repositories.rs
EOF

cat <<EOF > infrastructure/src/lib.rs
pub mod model;
pub mod repository;

mod repositories;
pub use repositories::RepositoriesImpl;
EOF
