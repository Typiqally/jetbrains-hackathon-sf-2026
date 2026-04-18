//! WP9 — end-to-end `lintropy init --with-skill`.

use std::fs;

use assert_cmd::Command;
use serde_json::Value;

fn lintropy() -> Command {
    Command::cargo_bin("lintropy").unwrap()
}

#[test]
fn init_with_skill_installs_skill_and_merges_settings_when_claude_present() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join(".claude")).unwrap();
    let preexisting = serde_json::json!({
        "hooks": {
            "PreToolUse": [
                { "matcher": "Bash", "hooks": [{ "type": "command", "command": "my-pre-hook" }] }
            ]
        },
        "other_user_setting": "keep-me"
    });
    fs::write(
        dir.path().join(".claude/settings.json"),
        serde_json::to_string_pretty(&preexisting).unwrap(),
    )
    .unwrap();

    lintropy()
        .current_dir(dir.path())
        .args(["init", "--with-skill"])
        .assert()
        .code(0);

    assert!(dir.path().join("lintropy.yaml").is_file());
    let root_cfg = fs::read_to_string(dir.path().join("lintropy.yaml")).unwrap();
    let parsed: serde_yaml::Value =
        serde_yaml::from_str(&root_cfg).expect("lintropy.yaml must parse as YAML");
    assert_eq!(parsed["version"].as_u64(), Some(1));

    let skill = dir.path().join(".claude/skills/lintropy/SKILL.md");
    assert!(skill.is_file(), "SKILL.md must be installed");
    let first_line = fs::read_to_string(&skill)
        .unwrap()
        .lines()
        .next()
        .unwrap()
        .to_string();
    assert!(
        first_line.starts_with("# version:"),
        "first line must carry `# version:` header, got: {first_line}"
    );

    let settings: Value = serde_json::from_str(
        &fs::read_to_string(dir.path().join(".claude/settings.json")).unwrap(),
    )
    .unwrap();
    // Unrelated user settings preserved.
    assert_eq!(settings["other_user_setting"], "keep-me");
    assert_eq!(settings["hooks"]["PreToolUse"][0]["matcher"], "Bash");
    // Lintropy PostToolUse entry merged in.
    let post = settings["hooks"]["PostToolUse"].as_array().unwrap();
    assert!(
        post.iter()
            .any(|entry| entry["matcher"] == "Write|Edit|NotebookEdit"
                && entry["hooks"][0]["command"] == "lintropy hook --agent claude-code"),
        "expected PostToolUse entry with matcher + lintropy hook command, got {post:?}"
    );
}
