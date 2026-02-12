use assert_cmd::Command;
use predicates::str::contains;
use tempfile::tempdir;
use std::fs;

#[test]
fn examples_lists_templates() {
    let mut cmd = Command::cargo_bin("dwf").unwrap();
    cmd.arg("examples")
        .assert()
        .success()
        .stdout(contains("rust-default"));
}

#[test]
fn init_creates_dwf_toml() {
    let dir = tempdir().unwrap();
    std::env::set_current_dir(dir.path()).unwrap();

    let mut cmd = Command::cargo_bin("dwf").unwrap();
    cmd.args(["init", "--template", "rust-default"])
        .assert()
        .success();

    let s = fs::read_to_string(dir.path().join("dwf.toml")).unwrap();
    assert!(s.contains("pipeline"));
}

#[test]
fn run_fails_without_dwf_toml() {
    let dir = tempdir().unwrap();
    std::env::set_current_dir(dir.path()).unwrap();

    let mut cmd = Command::cargo_bin("dwf").unwrap();
    cmd.args(["run", "fast"])
        .assert()
        .failure()
        .stderr(contains("dwf.toml not found"));
}
