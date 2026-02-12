use crate::config::Config;
use crate::metrics::RunRecord;

pub fn print_doctor(cfg: &Config, runs: &[RunRecord]) {
    println!("dwf doctor");
    println!("Config hints:");
    println!(
        "  clippy_deny_warnings: {}",
        cfg.pipeline.clippy_deny_warnings
    );
    println!(
        "  all_features_in_full: {}",
        cfg.pipeline.all_features_in_full
    );

    // Environment signals
    let rustc_wrapper = std::env::var("RUSTC_WRAPPER").ok();
    if rustc_wrapper.is_none() {
        println!("\nSuggestion: Consider enabling a compiler cache (e.g., sccache).");
        println!("  RUSTC_WRAPPER is not set.");
    } else {
        println!("\nDetected RUSTC_WRAPPER: {:?}", rustc_wrapper);
    }

    if runs.is_empty() {
        println!("\nNo recent runs found to analyze. Run `dwf run` a few times first.");
        return;
    }

    // Analyze recent timings
    let mut clippy_ms = Vec::new();
    let mut check_ms = Vec::new();
    let mut test_ms = Vec::new();

    for r in runs {
        for s in &r.steps {
            match s.name.as_str() {
                "clippy" => clippy_ms.push(s.duration_ms),
                "check" => check_ms.push(s.duration_ms),
                "test" => test_ms.push(s.duration_ms),
                _ => {}
            }
        }
    }

    let med_check = median(&check_ms);
    let med_clippy = median(&clippy_ms);
    let med_test = median(&test_ms);

    println!("\nRecent medians (ms):");
    if med_check > 0 {
        println!("  check : {}", med_check);
    }
    if med_clippy > 0 {
        println!("  clippy: {}", med_clippy);
    }
    if med_test > 0 {
        println!("  test  : {}", med_test);
    }

    if med_clippy > 0 && med_check > 0 && med_clippy > med_check.saturating_mul(2) {
        println!(
            "\nSuggestion: clippy is dominating; keep it after `cargo check` (already staged)."
        );
        println!("  Consider running clippy only in --full mode during early iteration.");
    }

    if med_test > 0 && med_test > (med_check.max(1)).saturating_mul(3) {
        println!("\nSuggestion: tests are dominating; rely on `dwf run fast` while iterating,");
        println!("  then confirm with `dwf run full` before pushing.");
    }

    // Failure stage distribution
    let mut fails = std::collections::BTreeMap::<String, usize>::new();
    for r in runs.iter().filter(|r| !r.ok) {
        let st = r
            .failure_stage
            .clone()
            .unwrap_or_else(|| "unknown".to_string());
        *fails.entry(st).or_insert(0) += 1;
    }
    if !fails.is_empty() {
        println!("\nFailure stage distribution (recent):");
        for (k, v) in fails {
            println!("  {:<8} {}", k, v);
        }
    }
}

fn median(v: &[u64]) -> u64 {
    if v.is_empty() {
        return 0;
    }
    let mut s = v.to_vec();
    s.sort_unstable();
    let mid = s.len() / 2;
    if s.len() % 2 == 1 {
        s[mid]
    } else {
        (s[mid - 1] / 2) + (s[mid] / 2) + ((s[mid - 1] % 2 + s[mid] % 2) / 2)
    }
}
