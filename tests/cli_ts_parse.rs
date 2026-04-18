//! Integration tests for `lintropy ts-parse`.

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn ts_parse_emits_rust_sexp() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("hello.rs");
    std::fs::write(&file, "fn main() {}\n").unwrap();
    Command::cargo_bin("lintropy")
        .unwrap()
        .args(["ts-parse", file.to_str().unwrap()])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("source_file"))
        .stdout(predicate::str::contains("function_item"));
}

#[test]
fn ts_parse_respects_lang_override() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("no_ext");
    std::fs::write(&file, "fn main() {}\n").unwrap();
    Command::cargo_bin("lintropy")
        .unwrap()
        .args(["ts-parse", file.to_str().unwrap(), "--lang", "rust"])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("source_file"));
}

#[test]
fn ts_parse_unknown_extension_lists_available_langs() {
    let tmp = tempfile::tempdir().unwrap();
    let file = tmp.path().join("foo.unknown");
    std::fs::write(&file, "hello").unwrap();
    let mut cmd = assert_cmd::Command::cargo_bin("lintropy").unwrap();
    cmd.arg("ts-parse").arg(&file);
    let assert = cmd.assert().failure();
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr).into_owned();
    assert!(
        stderr.contains("rust"),
        "error should list rust among available langs: {stderr}"
    );
}

#[test]
fn ts_parse_unknown_language_lists_available_langs() {
    let tmp = tempfile::tempdir().unwrap();
    let file = tmp.path().join("foo.txt");
    std::fs::write(&file, "hello").unwrap();
    let mut cmd = assert_cmd::Command::cargo_bin("lintropy").unwrap();
    cmd.arg("ts-parse").arg(&file).arg("--lang").arg("brainfuck");
    let assert = cmd.assert().failure();
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr).into_owned();
    assert!(stderr.contains("brainfuck"), "echo unknown name: {stderr}");
    assert!(stderr.contains("rust"), "list rust: {stderr}");
}
