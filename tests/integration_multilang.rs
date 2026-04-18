use assert_cmd::Command;
use tempfile::TempDir;

fn write(dir: &std::path::Path, rel: &str, contents: &str) {
    let path = dir.join(rel);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(path, contents).unwrap();
}

#[cfg(feature = "lang-typescript")]
#[test]
fn tsx_jsx_rule_matches_only_in_tsx_files() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    write(root, "lintropy.yaml", "version: 1\n");
    write(
        root,
        ".lintropy/no-raw-div.rule.yaml",
        r#"severity: warning
message: "no raw <div>"
language: typescript
query: |
  (jsx_element
    (jsx_opening_element (identifier) @name)
    (#eq? @name "div")) @m
"#,
    );
    write(root, "src/app.tsx", "const x = <div></div>;\n");
    write(root, "src/lib.ts", "const x: number = 1;\n");

    let mut cmd = Command::cargo_bin("lintropy").unwrap();
    cmd.current_dir(root).arg("check").arg("--format").arg("json");
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("app.tsx"), "tsx match missing: {stdout}");
    assert!(!stdout.contains("lib.ts"), "false positive on lib.ts: {stdout}");
}
