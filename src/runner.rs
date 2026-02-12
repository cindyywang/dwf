use crate::cli::Mode;
use crate::config::Config;
use crate::metrics::{RunRecord, StepRecord};
use anyhow::{Context, Result};
use std::process::Command;
use std::time::Instant;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

#[derive(Debug)]
struct CmdResult {
    ok: bool,
    code: Option<i32>,
    stderr: String,
    duration_ms: u64,
}

pub fn run_pipeline(cfg: &Config, mode: Mode) -> Result<RunRecord> {
    let start = Instant::now();
    let ts = OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| "unknown".to_string());

    let mut steps: Vec<StepRecord> = Vec::new();

    // 1) fmt
    let r = run_cmd(cfg, "fmt", cargo_fmt_cmd())?;
    steps.push(step_record("fmt", &r));
    if !r.ok {
        return Ok(finalize_run(ts, mode, start, steps));
    }

    // 2) check
    let r = run_cmd(cfg, "check", cargo_check_cmd())?;
    steps.push(step_record("check", &r));
    if !r.ok {
        return Ok(finalize_run(ts, mode, start, steps));
    }

    // 3) clippy
    let r = run_cmd(cfg, "clippy", cargo_clippy_cmd(cfg, mode))?;
    steps.push(step_record("clippy", &r));
    if !r.ok {
        return Ok(finalize_run(ts, mode, start, steps));
    }

    // 4) test (only in full)
    if mode == Mode::Full {
        let r = run_cmd(cfg, "test", cargo_test_cmd(cfg, mode))?;
        steps.push(step_record("test", &r));
        if !r.ok {
            return Ok(finalize_run(ts, mode, start, steps));
        }
    }

    Ok(finalize_run(ts, mode, start, steps))
}

fn finalize_run(ts: String, mode: Mode, start: Instant, steps: Vec<StepRecord>) -> RunRecord {
    let total_ms = start.elapsed().as_millis() as u64;

    // TTS: time until first failing step, else total
    let mut tts_ms = total_ms;
    let mut acc = 0u64;
    for s in &steps {
        acc = acc.saturating_add(s.duration_ms);
        if !s.ok {
            tts_ms = acc;
            break;
        }
    }

    let ok = steps.iter().all(|s| s.ok);
    let ttg_ms = if ok { Some(total_ms) } else { None };
    let failure_stage = steps.iter().find(|s| !s.ok).map(|s| s.name.clone());

    RunRecord {
        timestamp_rfc3339: ts,
        mode,
        ok,
        tts_ms,
        ttg_ms,
        total_ms,
        steps,
        failure_stage,
    }
}

fn step_record(name: &str, r: &CmdResult) -> StepRecord {
    StepRecord {
        name: name.to_string(),
        ok: r.ok,
        exit_code: r.code,
        duration_ms: r.duration_ms,
        stderr_excerpt: r.stderr.clone(),
    }
}

fn run_cmd(cfg: &Config, step_name: &str, mut cmd: Command) -> Result<CmdResult> {
    let t0 = Instant::now();
    let out = cmd
        .output()
        .with_context(|| format!("failed to execute step `{}`", step_name))?;
    let duration_ms = t0.elapsed().as_millis() as u64;

    let code = out.status.code();
    let ok = out.status.success();

    let stderr_raw = String::from_utf8_lossy(&out.stderr).to_string();
    let stderr = trim_lines(&stderr_raw, cfg.pipeline.stderr_max_lines);

    Ok(CmdResult {
        ok,
        code,
        stderr,
        duration_ms,
    })
}

fn cargo_fmt_cmd() -> Command {
    let mut c = Command::new("cargo");
    c.arg("fmt").arg("--all").arg("--check");
    c
}

fn cargo_check_cmd() -> Command {
    let mut c = Command::new("cargo");
    c.arg("check").arg("-q");
    c
}

fn cargo_clippy_cmd(cfg: &Config, mode: Mode) -> Command {
    let mut c = Command::new("cargo");
    c.arg("clippy").arg("--all-targets");
    if mode == Mode::Full && cfg.pipeline.all_features_in_full {
        c.arg("--all-features");
    }
    if cfg.pipeline.clippy_deny_warnings {
        c.arg("--").arg("-D").arg("warnings");
    }
    c
}

fn cargo_test_cmd(cfg: &Config, mode: Mode) -> Command {
    let mut c = Command::new("cargo");
    c.arg("test").arg("-q");
    if mode == Mode::Full && cfg.pipeline.all_features_in_full {
        c.arg("--all-features");
    }
    c
}

pub(crate) fn trim_lines(s: &str, max_lines: usize) -> String {
    if max_lines == 0 {
        return String::new();
    }
    let lines: Vec<&str> = s.lines().collect();
    if lines.len() <= max_lines {
        return s.to_string();
    }
    let mut out = String::new();
    for (i, line) in lines.iter().take(max_lines).enumerate() {
        out.push_str(line);
        if i + 1 < max_lines {
            out.push('\n');
        }
    }
    out.push_str("\n… (stderr truncated)\n");
    out
}

pub fn print_run_summary(run: &RunRecord) {
    println!("Mode: {:?} | ok: {}", run.mode, run.ok);
    for s in &run.steps {
        let status = if s.ok { "✅" } else { "❌" };
        println!(
            "  {} {:<6}  {} ms  exit={:?}",
            status, s.name, s.duration_ms, s.exit_code
        );
        if !s.ok && !s.stderr_excerpt.trim().is_empty() {
            println!("--- stderr (excerpt) ---");
            println!("{}", s.stderr_excerpt);
            println!("------------------------");
        }
    }
    println!("TTS: {} ms", run.tts_ms);
    match run.ttg_ms {
        Some(ms) => println!("TTG: {} ms", ms),
        None => println!("TTG: (not green)"),
    }
    println!("Total: {} ms", run.total_ms);
}

#[cfg(test)]
mod tests {
    use super::trim_lines;

    #[test]
    fn trims_to_max_lines() {
        let s = "a\nb\nc\nd\ne\n";
        let out = trim_lines(s, 3);

        assert!(out.contains("a\nb\nc"));
        assert!(out.contains("truncated"));
        assert!(!out.contains("d\ne"));
    }

    #[test]
    fn no_trim_when_short() {
        let s = "a\nb\n";
        let out = trim_lines(s, 5);
        assert_eq!(out, s);
    }

    #[test]
    fn zero_max_returns_empty() {
        let s = "hello\nworld";
        let out = trim_lines(s, 0);
        assert!(out.is_empty());
    }
}
