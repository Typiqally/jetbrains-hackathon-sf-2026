//! WP9 — end-to-end `lintropy hook` against a Claude-style payload.

use std::path::PathBuf;

use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::json;

fn lintropy() -> Command {
    Command::cargo_bin("lintropy").unwrap()
}

fn rust_demo() -> PathBuf {
    let manifest = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    PathBuf::from(manifest).join("examples/rust-demo")
}

#[test]
fn hook_exit_2_when_payload_points_at_triggering_file() {
    let demo = rust_demo();
    let payload = json!({
        "tool_input": { "file_path": demo.join("src/main.rs") }
    });

    lintropy()
        .current_dir(&demo)
        .args(["hook", "--fail-on", "warning"])
        .write_stdin(payload.to_string())
        .assert()
        .code(2)
        .stderr(predicate::str::contains("no-unwrap"));
}

#[test]
fn hook_exit_0_for_clean_file_scope() {
    let demo = rust_demo();
    let payload = json!({
        "tool_input": { "file_path": demo.join("Cargo.toml") }
    });

    lintropy()
        .current_dir(&demo)
        .arg("hook")
        .write_stdin(payload.to_string())
        .assert()
        .code(0)
        .stderr(predicate::str::is_empty());
}
