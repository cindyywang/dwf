use crate::cli::Mode;
use crate::config;
use crate::config::Config;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Write};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepRecord {
    pub name: String,
    pub ok: bool,
    pub exit_code: Option<i32>,
    pub duration_ms: u64,
    pub stderr_excerpt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunRecord {
    pub timestamp_rfc3339: String,
    pub mode: Mode,
    pub ok: bool,
    pub tts_ms: u64,
    pub ttg_ms: Option<u64>,
    pub total_ms: u64,
    pub steps: Vec<StepRecord>,
    pub failure_stage: Option<String>,
}

fn load_cfg_for_storage() -> Result<Config> {
    // This module may be called from report/doctor even if config missing.
    // But for simplicity (exam scope), require dwf.toml.
    crate::config::load_config()
}

pub fn append_run(run: &RunRecord) -> Result<()> {
    let cfg = load_cfg_for_storage()?;
    use std::path::Path;
    let dir = Path::new(&cfg.storage.dir);
    let file = dir.join(&cfg.storage.history_file);
    if let Some(parent) = file.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut f = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&file)
        .with_context(|| format!("open history file {:?}", file))?;

    let line = serde_json::to_string(run).context("serialize run to json")?;
    writeln!(f, "{}", line).context("write history line")?;
    Ok(())
}

pub fn load_last_runs(last: usize) -> Result<Vec<RunRecord>> {
    let cfg = load_cfg_for_storage()?;
    let (_dir, file) = config::storage_paths(&cfg);
    if !file.exists() {
        return Ok(Vec::new());
    }

    let f = OpenOptions::new()
        .read(true)
        .open(&file)
        .with_context(|| format!("open history file {:?}", file))?;
    let reader = BufReader::new(f);

    let mut runs: Vec<RunRecord> = Vec::new();
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        match serde_json::from_str::<RunRecord>(&line) {
            Ok(r) => runs.push(r),
            Err(_) => {
                // Ignore malformed lines (robustness)
            }
        }
    }

    if last == 0 || runs.is_empty() {
        return Ok(Vec::new());
    }

    // Keep only last N
    let start = runs.len().saturating_sub(last);
    Ok(runs[start..].to_vec())
}
