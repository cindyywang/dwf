use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

const CONFIG_FILE: &str = "dwf.toml";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub pipeline: PipelineConfig,
    pub storage: StorageConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineConfig {
    /// If true, add --all-features in full mode for clippy/test
    pub all_features_in_full: bool,

    /// Treat clippy warnings as errors (recommended for exams/CI)
    pub clippy_deny_warnings: bool,

    /// Trim stderr to this many lines for display
    pub stderr_max_lines: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Directory for history (relative to repo)
    pub dir: String,

    /// History file name (JSONL)
    pub history_file: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            pipeline: PipelineConfig {
                all_features_in_full: false,
                clippy_deny_warnings: true,
                stderr_max_lines: 80,
            },
            storage: StorageConfig {
                dir: ".dwf".to_string(),
                history_file: "history.jsonl".to_string(),
            },
        }
    }
}

pub fn init_config(force: bool) -> Result<()> {
    let p = Path::new(CONFIG_FILE);
    if p.exists() && !force {
        return Err(anyhow!(
            "dwf.toml already exists (use --force to overwrite)"
        ));
    }

    let cfg = Config::default();
    let toml_str = toml::to_string_pretty(&cfg).context("serialize config to TOML")?;
    fs::write(p, toml_str).context("write dwf.toml")?;
    Ok(())
}

pub fn load_config() -> Result<Config> {
    let p = Path::new(CONFIG_FILE);
    if !p.exists() {
        return Err(anyhow!(
            "dwf.toml not found. Run `dwf init` in the target repo first."
        ));
    }
    let s = fs::read_to_string(p).context("read dwf.toml")?;
    let cfg: Config = toml::from_str(&s).context("parse dwf.toml")?;
    Ok(cfg)
}

pub fn storage_paths(cfg: &Config) -> (std::path::PathBuf, std::path::PathBuf) {
    let dir = std::path::PathBuf::from(&cfg.storage.dir);
    let file = dir.join(&cfg.storage.history_file);
    (dir, file)
}
