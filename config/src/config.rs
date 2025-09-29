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
    pub title_max_len: usize,     // default 80 (80〜240) clamp
    pub body_max_len: usize,      // default 5000 (1000〜30000) clamp
    pub tag_max_len: usize,       // default 16 (8〜32) clamp
    pub category_max_len: usize,  // default 16 (8〜32) clamp
    pub max_tags: usize,          // default 6 (1〜100) clamp
    pub max_categories: usize,    // default 3 (1〜5) clamp
    pub template_dir: String,     // default "data/templates"
    pub template_content: String, // default "content.html"
    pub template_index: String,   // default "index.html"
    pub template_list: String,    // default "list.html"
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchConfig {
    pub dictionary_dir: String,        // default "data/dictionary"
    pub index_dir: String,             // default "output/.index"
    pub index_limit: usize,            // default 3000 (100〜10000) clamp
    pub search_limit: usize,           // default 1000 (100) clamp
    pub memory_budget_in_bytes: usize, // default 50_000_000 (10_000_000〜99_999_999)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SecurityConfig {
    pub issuer: String,    // default exe basename
    pub secret: String,    // default uuid v4
    pub expire: i64,       // seconds; default 86400; clamp 180..=90days
    pub user_file: String, // default "data/security/user.dat"

    pub lock_threshold: i64,  // default 3; clamp 1〜10
    pub lock_seconds: i64,    // seconds; default 3600; clamp 60〜60*60*24
    pub update_interval: i64, // seconds; default 1; clamp 1〜60
    pub allow_signup: bool,   // default false
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
                body_max_len: 5000,
                tag_max_len: 16,
                category_max_len: 16,
                max_tags: 6,
                max_categories: 3,
                template_dir: "data/templates".to_string(),
                template_content: "content.html".to_string(),
                template_index: "index.html".to_string(),
                template_list: "list.html".to_string(),
            },
            search: SearchConfig {
                dictionary_dir: "data/dictionary".to_string(),
                index_dir: "output/.index".to_string(),
                index_limit: 3000,
                search_limit: 1000,
                memory_budget_in_bytes: 50_000_000,
            },
            security: SecurityConfig {
                issuer: exe_name.clone(),
                secret: Uuid::new_v4().to_string(),
                expire: 60 * 60 * 24, // 1 day
                user_file: "data/security/user.dat".to_string(),
                lock_threshold: 3,
                lock_seconds: 60 * 60, // 1H
                update_interval: 1,
                allow_signup: false,
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
    title_max_len: Option<usize>,
    body_max_len: Option<usize>,
    tag_max_len: Option<usize>,
    category_max_len: Option<usize>,
    max_tags: Option<usize>,
    max_categories: Option<usize>,
    template_dir: Option<String>,
    template_content: Option<String>,
    template_index: Option<String>,
    template_list: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PartialSearchConfig {
    dictionary_dir: Option<String>,
    index_dir: Option<String>,
    max_index_count: Option<usize>,
    search_limit: Option<usize>,
    memory_budget_in_bytes: Option<usize>,
}

#[derive(Debug, Deserialize)]
struct PartialSecurityConfig {
    issuer: Option<String>,
    secret: Option<String>,
    expire: Option<i64>,
    user_file: Option<String>,
    lock_threshold: Option<i64>,
    lock_seconds: Option<i64>,
    update_interval: Option<i64>,
    allow_signup: Option<bool>,
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
            if let Some(v) = content.tag_max_len { self.content.tag_max_len = v; }
            if let Some(v) = content.category_max_len { self.content.category_max_len = v; }
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
            if let Some(v) = search.max_index_count { self.search.index_limit = v; }
            if let Some(v) = search.search_limit { self.search.search_limit = v; }
            if let Some(v) = search.memory_budget_in_bytes { self.search.memory_budget_in_bytes = v; }
        }
        if let Some(security) = p.security {
            if let Some(v) = security.issuer { self.security.issuer = v; }
            if let Some(v) = security.secret { self.security.secret = v; }
            if let Some(v) = security.expire { self.security.expire = v; }
            if let Some(v) = security.user_file { self.security.user_file = v; }
            if let Some(v) = security.lock_threshold { self.security.lock_threshold = v; }
            if let Some(v) = security.lock_seconds { self.security.lock_seconds = v; }
            if let Some(v) = security.update_interval { self.security.update_interval = v; }
            if let Some(v) = security.allow_signup { self.security.allow_signup = v; }
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
        // title
        let (tt_old, tt_new) = (
            self.content.title_max_len,
            clamp_usize(self.content.title_max_len, 80, 240),
        );
        if tt_old != tt_new {
            eprintln!(
                "title_max_len {} is out of range [80,160], rounded to {}.",
                tt_old, tt_new
            );
            self.content.title_max_len = tt_new;
        }

        // body
        let (bd_old, bd_new) = (
            self.content.body_max_len,
            clamp_usize(self.content.body_max_len, 1000, 30000),
        );
        if bd_old != bd_new {
            eprintln!(
                "body_max_len {} is out of range [100,5000], rounded to {}.",
                bd_old, bd_new
            );
            self.content.body_max_len = bd_new;
        }

        // tag
        let (tg_old, tg_new) = (
            self.content.tag_max_len,
            clamp_usize(self.content.tag_max_len, 8, 32),
        );

        if tg_old != tg_new {
            eprintln!(
                "tag_max_len {} is out of range [8,32], rounded to {}.",
                tg_old, tg_new
            );
            self.content.tag_max_len = tg_new;
        }

        // category
        let (ct_old, ct_new) = (
            self.content.category_max_len,
            clamp_usize(self.content.category_max_len, 8, 32),
        );
        if ct_old != ct_new {
            eprintln!(
                "category_max_len {} is out of range [8,32], rounded to {}.",
                ct_old, ct_new
            );
            self.content.category_max_len = ct_new;
        }

        // tag array size
        let (tag_old, tag_new) = (
            self.content.max_tags,
            clamp_usize(self.content.max_tags, 1, 100),
        );
        if tag_old != tag_new {
            eprintln!(
                "max_tags {} is out of range [1,100], rounded to {}.",
                tag_old, tag_new
            );
            self.content.max_tags = tag_new;
        }

        // categiory array size
        let (cat_old, cat_new) = (
            self.content.max_categories,
            clamp_usize(self.content.max_categories, 1, 5),
        );
        if cat_old != cat_new {
            eprintln!(
                "max_categories {} is out of range [1,5], rounded to {}.",
                cat_old, cat_new
            );
            self.content.max_categories = cat_new;
        }

        // index limit
        let (il_old, il_new) = (
            self.search.index_limit,
            clamp_usize(self.search.index_limit, 100, 10_000),
        );
        if il_old != il_new {
            eprintln!(
                "index_limit {} out of range [100,10000], rounded to {}.",
                il_old, il_new
            );
            self.search.index_limit = il_new;
        }

        // search limit
        let (sl_old, sl_new) = (
            self.search.search_limit,
            clamp_usize(self.search.search_limit, 100, 10_000),
        );
        if sl_old != sl_new {
            eprintln!(
                "search_limit {} out of range [100,10000], rounded to {}.",
                sl_old, sl_new
            );
            self.search.search_limit = sl_new;
        }

        // memory budget
        let (mb_old, mb_new) = (
            self.search.memory_budget_in_bytes,
            clamp_usize(self.search.memory_budget_in_bytes, 10_000_000, 99_999_999),
        );
        if mb_old != mb_new {
            eprintln!(
                "memory_budget_in_bytes {} out of range [10000000,9999999], rounded to {}.",
                mb_old, mb_new
            );
            self.search.memory_budget_in_bytes = mb_new;
        }

        // Security: expire clamp (180 ..= 90 days)
        let expire_clamp = clamp_i64(self.security.expire, 180, 60 * 60 * 24 * 90);
        if expire_clamp != self.security.expire {
            eprintln!(
                "security.expire {} is out of range [180,90days], rounded to {}.",
                self.security.expire, expire_clamp
            );
            self.security.expire = expire_clamp;
        }

        let lock_threshold_clamp = clamp_i64(self.security.lock_threshold, 1, 10);
        if lock_threshold_clamp != self.security.lock_threshold {
            eprintln!(
                "security.lock_threshold {} is out of range [1,10], rounded to {}.",
                self.security.lock_threshold, lock_threshold_clamp
            );
            self.security.lock_threshold = lock_threshold_clamp;
        }

        let lock_seconds_clamp = clamp_i64(self.security.lock_seconds, 60, 60 * 60 * 24);
        if lock_seconds_clamp != self.security.lock_seconds {
            eprintln!(
                "security.lock_seconds {} is out of range [60,1day], rounded to {}.",
                self.security.lock_seconds, lock_seconds_clamp
            );
            self.security.lock_seconds = lock_seconds_clamp;
        }

        let update_interval_clamp = clamp_i64(self.security.update_interval, 1, 60);
        if update_interval_clamp != self.security.update_interval {
            eprintln!(
                "security.update_interval {} is out of range [1,60], rounded to {}.",
                self.security.update_interval, update_interval_clamp
            );
            self.security.update_interval = update_interval_clamp;
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

        // security
        match (cli.allow_signup, cli.no_allow_signup) {
            (true, true) => {
                eprintln!(
                    "CLI: both --allow-signup and --no-allow-signup specified. --no-allow-signup will take precedence."
                );
                self.security.allow_signup = false;
            }
            (true, false) => self.security.allow_signup = true,
            (false, true) => self.security.allow_signup = false,
            (false, false) => {}
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

fn clamp_usize(v: usize, lo: usize, hi: usize) -> usize {
    if v < lo {
        lo
    } else if v > hi {
        hi
    } else {
        v
    }
}

fn clamp_i64(v: i64, lo: i64, hi: i64) -> i64 {
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

    #[arg(long)]
    pub allow_signup: bool,
    #[arg(long)]
    pub no_allow_signup: bool,
}
