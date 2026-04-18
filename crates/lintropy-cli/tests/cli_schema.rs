//! Integration tests for `lintropy schema`.

use assert_cmd::Command;

#[test]
fn schema_emits_parseable_json_with_properties() {
    let out = Command::cargo_bin("lintropy")
        .unwrap()
        .arg("schema")
        .assert()
        .code(0)
        .get_output()
        .stdout
        .clone();
    let parsed: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert!(parsed.as_object().unwrap().contains_key("properties"));
}
