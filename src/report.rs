use crate::metrics::RunRecord;

pub fn print_report(runs: &[RunRecord], last: usize) {
    if runs.is_empty() {
        println!("No runs found (expected .dwf/history.jsonl).");
        return;
    }

    let count = runs.len();
    let tts: Vec<u64> = runs.iter().map(|r| r.tts_ms).collect();
    let ttg: Vec<u64> = runs.iter().filter_map(|r| r.ttg_ms).collect();

    let avg_tts = avg(&tts);
    let med_tts = median(&tts);

    let avg_ttg = if ttg.is_empty() { None } else { Some(avg(&ttg)) };
    let med_ttg = if ttg.is_empty() { None } else { Some(median(&ttg)) };

    let ok_count = runs.iter().filter(|r| r.ok).count();
    let fail_count = count - ok_count;

    let mut fail_stage_counts = std::collections::BTreeMap::<String, usize>::new();
    for r in runs.iter().filter(|r| !r.ok) {
        let k = r.failure_stage.clone().unwrap_or_else(|| "unknown".to_string());
        *fail_stage_counts.entry(k).or_insert(0) += 1;
    }

    println!("Report (last {} requested, found {}):", last, count);
    println!("  ok: {} | failed: {}", ok_count, fail_count);
    println!("  TTS  avg: {} ms | median: {} ms", avg_tts, med_tts);

    match (avg_ttg, med_ttg) {
        (Some(a), Some(m)) => println!("  TTG  avg: {} ms | median: {} ms", a, m),
        _ => println!("  TTG  (no green runs in sample)"),
    }

    if !fail_stage_counts.is_empty() {
        println!("  Failure stages:");
        for (stage, n) in fail_stage_counts {
            println!("    {:<8} {}", stage, n);
        }
    }
}

fn avg(v: &[u64]) -> u64 {
    if v.is_empty() {
        return 0;
    }
    let sum: u128 = v.iter().map(|&x| x as u128).sum();
    (sum / v.len() as u128) as u64
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
        // even: average of two middle values (integer)
        (s[mid - 1] / 2) + (s[mid] / 2) + ((s[mid - 1] % 2 + s[mid] % 2) / 2)
    }
}
