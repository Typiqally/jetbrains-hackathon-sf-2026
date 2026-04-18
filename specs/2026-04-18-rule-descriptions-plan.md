# Rule descriptions Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add an optional `description` field to rules, surface it via
multi-line `lintropy rules` output with a `--group-by` flag, a new JSON
key, and a `description:` block in `lintropy explain`.

**Architecture:** Add one `Option<String>` field to the loader's raw and
resolved rule structs (`crates/lintropy-core/src/config.rs`). Rebuild the
`lintropy rules` text renderer as a multi-line formatter with three
grouping modes. Keep `lintropy rules --format json` a flat array; add one
new `description` key per entry. `lintropy explain` gains one conditional
block. The loader stays permissive (field is optional); SKILL.md guidance
is prescriptive (already landed).

**Tech Stack:** Rust stable (workspace is on 1.95.0), `clap`, `serde`,
`serde_yaml`, `schemars`, `assert_cmd`, `predicates`, `tempfile`,
`serde_json` (already in `[dev-dependencies]`).

**Spec:** `specs/2026-04-18-rule-descriptions.md`

---

## File Structure

Files to touch:

- **Modify** `crates/lintropy-core/src/config.rs`
  - Add `description: Option<String>` to `RawRule` (YAML-facing deserialise struct).
  - Add `description: Option<String>` to `RuleConfig` (resolved public struct).
  - In `build_rule`, copy field and normalise `Some("")` → `None`.
  - Extend `#[cfg(test)] mod tests` with four new unit tests.

- **Modify** `crates/lintropy-cli/src/cli.rs`
  - Add `GroupBy` enum (`None`, `Language`, `Tag`) with `ValueEnum`.
  - Add `--group-by` field to `RulesArgs`, defaulting to `GroupBy::None`.

- **Modify** `crates/lintropy-cli/src/commands/rules.rs`
  - Replace one-line text renderer with multi-line renderer.
  - Add grouping logic for `language` and `tag` modes.
  - Add `description` key to `rule_to_json`.
  - Reject `--group-by <non-none>` combined with `--format json`.

- **Modify** `crates/lintropy-cli/src/commands/explain.rs`
  - Insert `description:` block between `message` and `query`.

- **Create** `crates/lintropy-cli/tests/common/describe.rs`
  - Dedicated fixture builder for multi-rule description tests.
  - Exports `DescribeFixture::new()` with 4 rules spanning described/undescribed × tagged/untagged × Rust/Python/no-language.

- **Modify** `crates/lintropy-cli/tests/common/mod.rs`
  - Add `pub mod describe;` line so the new fixture is reachable from integration-test files.

- **Create** `crates/lintropy-cli/tests/cli_rules_describe.rs`
  - Integration tests for the new `rules` surface (text, JSON, grouping, error paths).

- **Create** `crates/lintropy-cli/tests/cli_explain_describe.rs`
  - Integration tests for `explain` description block.

- **Modify** `.lintropy/no-unwrap.rule.yaml`, `.lintropy/no-todo.rule.yaml`, `.lintropy/no-dbg.rule.yaml`
  - Backfill `description`.

- **Possibly modify** `crates/lintropy-cli/tests/cli_rules.rs`
  - Existing assertions (`no-unwrap`, `[warning]`) still hold under new output, but verify after Task 3 lands. No changes expected unless an assertion breaks.

---

## Task 1: Add `description` field to core config schema

**Files:**
- Modify: `crates/lintropy-core/src/config.rs`
- Test: `crates/lintropy-core/src/config.rs` (same file, `#[cfg(test)] mod tests`)

- [ ] **Step 1: Add four failing unit tests**

Open `crates/lintropy-core/src/config.rs`. At the bottom of the existing `#[cfg(test)] mod tests` block (currently ending around line 530), add these tests:

```rust
    use std::io::Write;

    fn write_fixture(dir: &std::path::Path, root: &str, rule: Option<(&str, &str)>) {
        std::fs::write(dir.join("lintropy.yaml"), root).unwrap();
        if let Some((filename, content)) = rule {
            let lintropy_dir = dir.join(".lintropy");
            std::fs::create_dir_all(&lintropy_dir).unwrap();
            std::fs::write(lintropy_dir.join(filename), content).unwrap();
        }
    }

    #[test]
    fn description_roundtrips_from_single_rule_file() {
        let tmp = tempfile::tempdir().unwrap();
        let rule = r#"language: rust
severity: warning
description: "Flags bare .unwrap() calls."
message: "no unwrap"
query: |
  ((identifier) @match (#eq? @match "foo"))
"#;
        write_fixture(
            tmp.path(),
            "version: 1\n",
            Some(("no-unwrap.rule.yaml", rule)),
        );
        let config = Config::load_from_root(tmp.path()).unwrap();
        assert_eq!(config.rules.len(), 1);
        assert_eq!(
            config.rules[0].description.as_deref(),
            Some("Flags bare .unwrap() calls.")
        );
    }

    #[test]
    fn description_absent_resolves_to_none() {
        let tmp = tempfile::tempdir().unwrap();
        let rule = r#"language: rust
severity: warning
message: "no unwrap"
query: |
  ((identifier) @match (#eq? @match "foo"))
"#;
        write_fixture(
            tmp.path(),
            "version: 1\n",
            Some(("no-unwrap.rule.yaml", rule)),
        );
        let config = Config::load_from_root(tmp.path()).unwrap();
        assert_eq!(config.rules.len(), 1);
        assert!(config.rules[0].description.is_none());
    }

    #[test]
    fn description_empty_string_normalises_to_none() {
        let tmp = tempfile::tempdir().unwrap();
        let rule = r#"language: rust
severity: warning
description: ""
message: "no unwrap"
query: |
  ((identifier) @match (#eq? @match "foo"))
"#;
        write_fixture(
            tmp.path(),
            "version: 1\n",
            Some(("no-unwrap.rule.yaml", rule)),
        );
        let config = Config::load_from_root(tmp.path()).unwrap();
        assert!(config.rules[0].description.is_none());
    }

    #[test]
    fn description_multiline_preserves_newlines() {
        let tmp = tempfile::tempdir().unwrap();
        let rule = r#"language: rust
severity: warning
description: |
  line one
  line two
message: "no unwrap"
query: |
  ((identifier) @match (#eq? @match "foo"))
"#;
        write_fixture(
            tmp.path(),
            "version: 1\n",
            Some(("no-unwrap.rule.yaml", rule)),
        );
        let config = Config::load_from_root(tmp.path()).unwrap();
        assert_eq!(
            config.rules[0].description.as_deref(),
            Some("line one\nline two\n")
        );
    }

    #[test]
    fn json_schema_exposes_description_property() {
        let schema = Config::json_schema();
        let schema_str = serde_json::to_string(&schema).unwrap();
        assert!(
            schema_str.contains("\"description\""),
            "expected `description` property in JSON schema, got: {schema_str}"
        );
    }
```

Note: the `use std::io::Write;` line can be removed if you don't end up using it; included above for the patterns. `tempfile` is already in `[dev-dependencies]` of `lintropy-core` (verify via `grep tempfile crates/lintropy-core/Cargo.toml`; if not, add it). `serde_json` is already a regular dependency.

- [ ] **Step 2: Verify dev-deps exist**

Run: `grep -E "tempfile|serde_json" crates/lintropy-core/Cargo.toml`

Expected: `tempfile` under `[dev-dependencies]`, `serde_json` under regular `[dependencies]`. If `tempfile` is missing, add it:

```toml
[dev-dependencies]
tempfile = "3"
```

- [ ] **Step 3: Run tests to confirm they fail**

Run: `cargo test -p lintropy-core description`

Expected: compile error — `RuleConfig` has no field `description`. Five tests fail to compile.

- [ ] **Step 4: Add `description` field to `RawRule`**

In `crates/lintropy-core/src/config.rs`, around line 155 (inside `struct RawRule`, after the `fix` field), add:

```rust
    #[serde(default)]
    description: Option<String>,
```

- [ ] **Step 5: Add `description` field to `RuleConfig`**

Around line 58 of the same file, inside `pub struct RuleConfig`, after `pub source_path: PathBuf,`, add:

```rust
    pub description: Option<String>,
```

- [ ] **Step 6: Wire field through `build_rule`**

In `build_rule`, right after `let rule_id = RuleId::new(id.clone());` (around line 288), add:

```rust
    let description = raw.description.clone().filter(|s| !s.is_empty());
```

(We `.clone()` rather than move so the rest of `raw` stays usable at the struct literal below. The description string is tiny, so a clone is fine.)

Then find the final `Ok(RuleConfig { ... })` block (around line 333) and add the field to the struct literal:

```rust
    Ok(RuleConfig {
        id: rule_id,
        severity,
        message: raw.message,
        include: raw.include,
        exclude: raw.exclude,
        tags: raw.tags,
        docs_url: raw.docs_url,
        language,
        kind,
        fix: raw.fix,
        source_path,
        description,
    })
```

Note: do **not** trim. Authors control whitespace (spec §3.2). The only normalisation is `Some("")` → `None`.

- [ ] **Step 7: Run tests to confirm they pass**

Run: `cargo test -p lintropy-core description`

Expected: five tests pass.

Also run the full core test suite to catch unrelated regressions:

```bash
cargo test -p lintropy-core
```

Expected: all green.

- [ ] **Step 8: Commit**

```bash
git add crates/lintropy-core/src/config.rs crates/lintropy-core/Cargo.toml
git commit -m "feat(core): add optional description field to rule config

Description is optional in YAML, preserved verbatim (newlines included),
and empty strings are normalised to None at build time. Schema auto-picks
up the new field via schemars.
"
```

---

## Task 2: Add `GroupBy` enum and `--group-by` flag to CLI

**Files:**
- Modify: `crates/lintropy-cli/src/cli.rs`

- [ ] **Step 1: Add the enum**

In `crates/lintropy-cli/src/cli.rs`, after the `OutputFormat` enum (around line 52), add:

```rust
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, ValueEnum)]
pub enum GroupBy {
    /// Flat list sorted by id (default).
    #[default]
    None,
    /// Group by rule language.
    Language,
    /// Group by the rule's first tag.
    Tag,
}
```

- [ ] **Step 2: Add the flag to `RulesArgs`**

Replace the existing `RulesArgs` struct (around line 98) with:

```rust
#[derive(Debug, Args)]
pub struct RulesArgs {
    /// Emit the list as JSON.
    #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
    pub format: OutputFormat,

    /// Group text output by language or first tag. Text format only.
    #[arg(long = "group-by", value_enum, default_value_t = GroupBy::None)]
    pub group_by: GroupBy,

    /// Override config discovery.
    #[arg(long, value_name = "PATH")]
    pub config: Option<PathBuf>,
}
```

- [ ] **Step 3: Confirm it compiles**

Run: `cargo build -p lintropy-cli`

Expected: clean build. (`rules.rs` does not yet read `group_by`, but the field exists with a default, so it compiles.)

- [ ] **Step 4: Commit**

```bash
git add crates/lintropy-cli/src/cli.rs
git commit -m "feat(cli): add --group-by flag to lintropy rules"
```

---

## Task 3: Rewrite `lintropy rules` text renderer (flat, multi-line)

**Files:**
- Modify: `crates/lintropy-cli/src/commands/rules.rs`
- Create: `crates/lintropy-cli/tests/common/describe.rs`
- Modify: `crates/lintropy-cli/tests/common/mod.rs`
- Create: `crates/lintropy-cli/tests/cli_rules_describe.rs`

- [ ] **Step 1: Create the test fixture builder**

Create `crates/lintropy-cli/tests/common/describe.rs`:

```rust
//! Multi-rule fixture for description + grouping tests.

#![allow(dead_code)]

use std::fs;
use std::path::Path;

use tempfile::TempDir;

/// Root config — minimal; rules live in `.lintropy/`.
const ROOT: &str = "version: 1\n";

/// Described + tagged Rust rule.
const NO_UNWRAP: &str = r#"language: rust
severity: warning
description: Flags `.unwrap()` on Result/Option.
tags: ["reliability", "rust"]
message: "no unwrap"
query: |
  (call_expression
    function: (field_expression
      field: (field_identifier) @m)
    (#eq? @m "unwrap")) @match
"#;

/// Described + untagged Python rule.
const NO_PRINT: &str = r#"language: python
severity: info
description: |
  Bans stray print() calls from shipped modules.
  Leave them to tests and scripts.
message: "no print"
query: |
  (call
    function: (identifier) @f
    (#eq? @f "print")) @match
"#;

/// Undescribed + tagged Rust rule.
const NO_DBG: &str = r#"language: rust
severity: error
tags: ["noise"]
message: "no dbg"
query: |
  (macro_invocation
    macro: (identifier) @n
    (#eq? @n "dbg")) @match
"#;

/// Undescribed + untagged, no language (so relies on match rule in phase 2).
/// Phase 1 loader rejects rules without `query`/`forbid`/`require`, so we
/// give it a minimal Rust query and simply leave `tags` empty.
const BARE: &str = r#"language: rust
severity: info
message: "bare"
query: |
  ((identifier) @match (#eq? @match "zzzz_unlikely"))
"#;

pub struct DescribeFixture {
    pub dir: TempDir,
}

impl DescribeFixture {
    pub fn new() -> Self {
        let dir = tempfile::tempdir().expect("create tempdir");
        fs::write(dir.path().join("lintropy.yaml"), ROOT).unwrap();
        let rules = dir.path().join(".lintropy");
        fs::create_dir_all(&rules).unwrap();
        fs::write(rules.join("no-unwrap.rule.yaml"), NO_UNWRAP).unwrap();
        fs::write(rules.join("no-print.rule.yaml"), NO_PRINT).unwrap();
        fs::write(rules.join("no-dbg.rule.yaml"), NO_DBG).unwrap();
        fs::write(rules.join("bare.rule.yaml"), BARE).unwrap();
        Self { dir }
    }

    pub fn path(&self) -> &Path {
        self.dir.path()
    }
}
```

Note: the spec's fourth fixture ("no-language") is relaxed here because phase 1 requires `query` → `language`. We keep the intent (a minimal rule with no tags, no description) and use a harmless Rust query. Coverage of `language == None` is exercised via future match rules; the JSON-null assertion for `language` stays testable because non-query rules would break at config-load time anyway.

Correction: the `tags` field on `bare` is omitted entirely here (no `tags:` key) so that the "untagged" grouping bucket is populated by at least one rule, and so we can assert that descriptionless rules produce `"description": null`.

- [ ] **Step 2: Expose the new fixture from `tests/common/mod.rs`**

Open `crates/lintropy-cli/tests/common/mod.rs`. At the top of the file (before or after the existing imports), add:

```rust
pub mod describe;
```

- [ ] **Step 3: Write the failing rules text test**

Create `crates/lintropy-cli/tests/cli_rules_describe.rs`:

```rust
//! Integration tests for `lintropy rules` description + grouping.

mod common;

use assert_cmd::Command;
use common::describe::DescribeFixture;
use predicates::prelude::*;

fn run_rules(fx: &DescribeFixture, args: &[&str]) -> assert_cmd::assert::Assert {
    let mut cmd = Command::cargo_bin("lintropy").unwrap();
    cmd.current_dir(fx.path()).arg("rules");
    for a in args {
        cmd.arg(a);
    }
    cmd.assert()
}

#[test]
fn rules_text_default_shows_description_line() {
    let fx = DescribeFixture::new();
    run_rules(&fx, &[])
        .code(0)
        .stdout(predicate::str::contains("no-unwrap"))
        .stdout(predicate::str::contains(
            "Flags `.unwrap()` on Result/Option.",
        ))
        .stdout(predicate::str::contains("tags: reliability, rust"))
        .stdout(predicate::str::contains(
            "source: .lintropy/no-unwrap.rule.yaml",
        ));
}

#[test]
fn rules_text_hides_description_when_absent() {
    let fx = DescribeFixture::new();
    let out = run_rules(&fx, &[]).code(0).get_output().stdout.clone();
    let text = String::from_utf8(out).unwrap();

    // The no-dbg rule has no description.
    let idx = text
        .find("no-dbg")
        .expect("no-dbg rule should appear in output");
    // The next rule header in sorted order is `no-print` or `no-unwrap`;
    // between the no-dbg header and the next header there must not be a
    // description body line.
    let after = &text[idx..];
    let next_blank = after.find("\n\n").unwrap_or(after.len());
    let block = &after[..next_blank];
    assert!(
        !block.contains("Flags"),
        "no-dbg block should have no description, got:\n{block}"
    );
}
```

- [ ] **Step 4: Run the test to confirm it fails**

Run: `cargo test -p lintropy-cli --test cli_rules_describe rules_text_default_shows_description_line`

Expected: fails. Current output is single-line; description text is not printed.

- [ ] **Step 5: Replace the text renderer in `commands/rules.rs`**

Open `crates/lintropy-cli/src/commands/rules.rs`. Replace the entire file contents with:

```rust
//! `lintropy rules` — list every loaded rule.

use lintropy_core::{Config, RuleConfig, RuleKind, Severity};
use serde_json::json;

use crate::cli::{GroupBy, OutputFormat, RulesArgs};
use crate::commands::{load_config, print_warnings};
use crate::exit::{CliError, EXIT_OK};

const WRAP_WIDTH: usize = 100;
const INDENT: &str = "  ";

pub fn run(args: RulesArgs) -> Result<u8, CliError> {
    let config = load_config(args.config.as_deref())?;
    print_warnings(&config);

    match args.format {
        OutputFormat::Text => {
            print_text(&config, args.group_by);
            Ok(EXIT_OK)
        }
        OutputFormat::Json => {
            if !matches!(args.group_by, GroupBy::None) {
                return Err(CliError::user(
                    "--group-by only applies to text format",
                ));
            }
            print_json(&config)?;
            Ok(EXIT_OK)
        }
    }
}

fn print_text(config: &Config, group_by: GroupBy) {
    let mut rules: Vec<&RuleConfig> = config.rules.iter().collect();
    rules.sort_by(|a, b| a.id.as_str().cmp(b.id.as_str()));

    match group_by {
        GroupBy::None => {
            print_rule_block(&rules);
        }
        GroupBy::Language => {
            let groups = group_by_language(&rules);
            for (label, group) in groups {
                print_group_header(&label);
                print_rule_block(&group);
                println!();
            }
        }
        GroupBy::Tag => {
            let groups = group_by_first_tag(&rules);
            for (label, group) in groups {
                print_group_header(&label);
                print_rule_block(&group);
                println!();
            }
        }
    }
}

fn print_group_header(label: &str) {
    println!("{label}");
    println!("{}", "-".repeat(label.chars().count()));
}

fn print_rule_block(rules: &[&RuleConfig]) {
    if rules.is_empty() {
        return;
    }
    let id_width = rules.iter().map(|r| r.id.as_str().len()).max().unwrap_or(0);
    let sev_width = rules
        .iter()
        .map(|r| severity_label(r.severity).len() + 2)
        .max()
        .unwrap_or(0);

    for (i, rule) in rules.iter().enumerate() {
        if i > 0 {
            println!();
        }
        let sev = format!("[{}]", severity_label(rule.severity));
        let lang = rule.language.map(|l| l.name()).unwrap_or("");
        println!(
            "{:id_w$}  {:sev_w$}  {}",
            rule.id.as_str(),
            sev,
            lang,
            id_w = id_width,
            sev_w = sev_width
        );
        if let Some(desc) = &rule.description {
            for line in wrap_description(desc) {
                println!("{INDENT}{line}");
            }
        }
        if !rule.tags.is_empty() {
            println!("{INDENT}tags: {}", rule.tags.join(", "));
        }
        if let Some(url) = &rule.docs_url {
            println!("{INDENT}docs: {url}");
        }
        println!("{INDENT}source: {}", rule.source_path.display());
    }
}

fn group_by_language<'a>(rules: &[&'a RuleConfig]) -> Vec<(String, Vec<&'a RuleConfig>)> {
    use std::collections::BTreeMap;
    let mut named: BTreeMap<String, Vec<&'a RuleConfig>> = BTreeMap::new();
    let mut anon: Vec<&'a RuleConfig> = Vec::new();
    for r in rules {
        match r.language {
            Some(lang) => named.entry(lang.name().to_string()).or_default().push(r),
            None => anon.push(*r),
        }
    }
    let mut out: Vec<(String, Vec<&'a RuleConfig>)> = named.into_iter().collect();
    if !anon.is_empty() {
        out.push(("(any)".to_string(), anon));
    }
    out
}

fn group_by_first_tag<'a>(rules: &[&'a RuleConfig]) -> Vec<(String, Vec<&'a RuleConfig>)> {
    use std::collections::BTreeMap;
    let mut named: BTreeMap<String, Vec<&'a RuleConfig>> = BTreeMap::new();
    let mut untagged: Vec<&'a RuleConfig> = Vec::new();
    for r in rules {
        match r.tags.first() {
            Some(t) => named.entry(t.clone()).or_default().push(r),
            None => untagged.push(*r),
        }
    }
    let mut out: Vec<(String, Vec<&'a RuleConfig>)> = named.into_iter().collect();
    if !untagged.is_empty() {
        out.push(("(untagged)".to_string(), untagged));
    }
    out
}

fn wrap_description(text: &str) -> Vec<String> {
    // Hard newlines in the source are preserved as paragraph breaks; within a
    // paragraph, soft-wrap on whitespace at WRAP_WIDTH.
    let mut out = Vec::new();
    for line in text.lines() {
        if line.is_empty() {
            out.push(String::new());
            continue;
        }
        let mut current = String::new();
        for word in line.split_whitespace() {
            if current.is_empty() {
                current.push_str(word);
            } else if current.len() + 1 + word.len() <= WRAP_WIDTH {
                current.push(' ');
                current.push_str(word);
            } else {
                out.push(std::mem::take(&mut current));
                current.push_str(word);
            }
        }
        if !current.is_empty() {
            out.push(current);
        }
    }
    out
}

fn print_json(config: &Config) -> Result<(), CliError> {
    let array: Vec<_> = config.rules.iter().map(rule_to_json).collect();
    let json = serde_json::to_string_pretty(&array)
        .map_err(|err| CliError::internal(format!("json: {err}")))?;
    println!("{json}");
    Ok(())
}

fn rule_to_json(rule: &RuleConfig) -> serde_json::Value {
    let kind = match &rule.kind {
        RuleKind::Query(_) => "query",
        RuleKind::Match(_) => "match",
    };
    json!({
        "id": rule.id.as_str(),
        "severity": severity_label(rule.severity),
        "language": rule.language.map(|l| l.name()),
        "kind": kind,
        "description": rule.description,
        "source_path": rule.source_path.display().to_string(),
        "tags": rule.tags,
        "docs_url": rule.docs_url,
        "include": rule.include,
        "exclude": rule.exclude,
    })
}

fn severity_label(severity: Severity) -> &'static str {
    match severity {
        Severity::Error => "error",
        Severity::Warning => "warning",
        Severity::Info => "info",
    }
}
```

- [ ] **Step 6: Run the failing test — expect pass now**

Run: `cargo test -p lintropy-cli --test cli_rules_describe`

Expected: both tests pass.

- [ ] **Step 7: Sanity-check that the existing `cli_rules.rs` test still passes**

Run: `cargo test -p lintropy-cli --test cli_rules`

Expected: both original tests still pass. The assertion
`predicate::str::contains("no-unwrap")` and
`predicate::str::contains("[warning]")` still hold under the new
multi-line output.

- [ ] **Step 8: Commit**

```bash
git add \
  crates/lintropy-cli/src/commands/rules.rs \
  crates/lintropy-cli/tests/common/mod.rs \
  crates/lintropy-cli/tests/common/describe.rs \
  crates/lintropy-cli/tests/cli_rules_describe.rs
git commit -m "feat(cli): multi-line rules output with description

Rewrites the `lintropy rules` text renderer to emit one header line
plus indented body lines (description, tags, docs, source). Absent
descriptions are elided. Grouping and JSON changes land in follow-up
commits.
"
```

---

## Task 4: Add `description` key to `rules --format json`

**Files:**
- Modify: `crates/lintropy-cli/tests/cli_rules_describe.rs`

Note: the code change already landed in Task 3 (`rule_to_json` already includes the `description` key). This task only adds the JSON assertions.

- [ ] **Step 1: Add two failing JSON tests**

Append to `crates/lintropy-cli/tests/cli_rules_describe.rs`:

```rust
fn json_output(fx: &DescribeFixture, args: &[&str]) -> serde_json::Value {
    let mut cmd = Command::cargo_bin("lintropy").unwrap();
    cmd.current_dir(fx.path()).arg("rules");
    for a in args {
        cmd.arg(a);
    }
    let out = cmd.assert().code(0).get_output().stdout.clone();
    serde_json::from_slice(&out).expect("valid JSON")
}

#[test]
fn rules_json_description_null_when_absent() {
    let fx = DescribeFixture::new();
    let arr = json_output(&fx, &["--format", "json"]);
    let arr = arr.as_array().unwrap();
    let dbg_rule = arr
        .iter()
        .find(|o| o["id"] == "no-dbg")
        .expect("no-dbg entry");
    assert_eq!(dbg_rule["description"], serde_json::Value::Null);
}

#[test]
fn rules_json_description_string_when_present() {
    let fx = DescribeFixture::new();
    let arr = json_output(&fx, &["--format", "json"]);
    let arr = arr.as_array().unwrap();
    let unwrap_rule = arr
        .iter()
        .find(|o| o["id"] == "no-unwrap")
        .expect("no-unwrap entry");
    assert_eq!(
        unwrap_rule["description"],
        "Flags `.unwrap()` on Result/Option."
    );
}
```

- [ ] **Step 2: Run**

Run: `cargo test -p lintropy-cli --test cli_rules_describe rules_json_description`

Expected: both pass (the production code was already written in Task 3; these tests lock the contract).

- [ ] **Step 3: Commit**

```bash
git add crates/lintropy-cli/tests/cli_rules_describe.rs
git commit -m "test(cli): lock description key in rules JSON output"
```

---

## Task 5: `--group-by language`

**Files:**
- Modify: `crates/lintropy-cli/tests/cli_rules_describe.rs`

No production-code change — Task 3's renderer already handles grouping. This task adds the test.

- [ ] **Step 1: Add failing test**

Append to `cli_rules_describe.rs`:

```rust
#[test]
fn rules_text_group_by_language_buckets_and_orders() {
    let fx = DescribeFixture::new();
    let out = run_rules(&fx, &["--group-by", "language"])
        .code(0)
        .get_output()
        .stdout
        .clone();
    let text = String::from_utf8(out).unwrap();

    let py_idx = text.find("python\n------").expect("python group header");
    let rs_idx = text.find("rust\n----").expect("rust group header");
    assert!(py_idx < rs_idx, "python should group before rust (alphabetical)");

    // Every rule should still appear.
    for id in ["no-unwrap", "no-print", "no-dbg", "bare"] {
        assert!(text.contains(id), "missing {id} in grouped output");
    }
}
```

- [ ] **Step 2: Run**

Run: `cargo test -p lintropy-cli --test cli_rules_describe rules_text_group_by_language`

Expected: pass.

- [ ] **Step 3: Commit**

```bash
git add crates/lintropy-cli/tests/cli_rules_describe.rs
git commit -m "test(cli): --group-by language buckets by language and sorts"
```

---

## Task 6: `--group-by tag`

**Files:**
- Modify: `crates/lintropy-cli/tests/cli_rules_describe.rs`

- [ ] **Step 1: Add failing test**

Append:

```rust
#[test]
fn rules_text_group_by_tag_first_tag_wins_untagged_last() {
    let fx = DescribeFixture::new();
    let out = run_rules(&fx, &["--group-by", "tag"])
        .code(0)
        .get_output()
        .stdout
        .clone();
    let text = String::from_utf8(out).unwrap();

    // Expected groups (alphabetical by first tag, then untagged bucket):
    //   noise         -> no-dbg
    //   reliability   -> no-unwrap   (first tag is "reliability", not "rust")
    //   (untagged)    -> no-print, bare
    let noise_idx = text.find("noise\n-----").expect("noise group header");
    let reliability_idx = text
        .find("reliability\n-----------")
        .expect("reliability group header");
    let untagged_idx = text
        .find("(untagged)\n----------")
        .expect("(untagged) group header");

    assert!(noise_idx < reliability_idx);
    assert!(reliability_idx < untagged_idx);

    // no-unwrap appears only once even though it also has a "rust" tag.
    assert_eq!(text.matches("no-unwrap").count(), 1);

    // No stray "rust" group header (would mean we double-bucketed).
    assert!(!text.contains("rust\n----"));
}
```

- [ ] **Step 2: Run**

Run: `cargo test -p lintropy-cli --test cli_rules_describe rules_text_group_by_tag`

Expected: pass.

- [ ] **Step 3: Commit**

```bash
git add crates/lintropy-cli/tests/cli_rules_describe.rs
git commit -m "test(cli): --group-by tag uses first tag, untagged last"
```

---

## Task 7: Reject `--group-by <non-none> --format json`

**Files:**
- Modify: `crates/lintropy-cli/tests/cli_rules_describe.rs`

The rejection branch is already in place (Task 3's `run` fn returns `CliError::user`). This task adds the test.

- [ ] **Step 1: Add failing test**

Append:

```rust
#[test]
fn rules_rejects_group_by_with_json_format() {
    let fx = DescribeFixture::new();
    Command::cargo_bin("lintropy")
        .unwrap()
        .current_dir(fx.path())
        .args(["rules", "--format", "json", "--group-by", "language"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "--group-by only applies to text format",
        ));
}
```

- [ ] **Step 2: Run**

Run: `cargo test -p lintropy-cli --test cli_rules_describe rules_rejects_group_by`

Expected: pass. Exit code non-zero, stderr contains the user-error message.

- [ ] **Step 3: Commit**

```bash
git add crates/lintropy-cli/tests/cli_rules_describe.rs
git commit -m "test(cli): reject --group-by together with --format json"
```

---

## Task 8: `lintropy explain` prints description block

**Files:**
- Modify: `crates/lintropy-cli/src/commands/explain.rs`
- Create: `crates/lintropy-cli/tests/cli_explain_describe.rs`

- [ ] **Step 1: Write failing tests**

Create `crates/lintropy-cli/tests/cli_explain_describe.rs`:

```rust
//! Integration tests for `lintropy explain` description handling.

mod common;

use assert_cmd::Command;
use common::describe::DescribeFixture;
use predicates::prelude::*;

#[test]
fn explain_prints_description_block_when_present() {
    let fx = DescribeFixture::new();
    Command::cargo_bin("lintropy")
        .unwrap()
        .current_dir(fx.path())
        .args(["explain", "no-unwrap"])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("description:"))
        .stdout(predicate::str::contains(
            "Flags `.unwrap()` on Result/Option.",
        ));
}

#[test]
fn explain_omits_description_block_when_absent() {
    let fx = DescribeFixture::new();
    let out = Command::cargo_bin("lintropy")
        .unwrap()
        .current_dir(fx.path())
        .args(["explain", "no-dbg"])
        .assert()
        .code(0)
        .get_output()
        .stdout
        .clone();
    let text = String::from_utf8(out).unwrap();
    assert!(
        !text.contains("description:"),
        "no-dbg has no description, but `description:` header appeared:\n{text}"
    );
}
```

- [ ] **Step 2: Run to confirm they fail**

Run: `cargo test -p lintropy-cli --test cli_explain_describe`

Expected: `explain_prints_description_block_when_present` fails (no
`description:` text yet). The "omits" test may currently pass
vacuously — that is fine; it becomes meaningful after the change.

- [ ] **Step 3: Modify `commands/explain.rs`**

Open `crates/lintropy-cli/src/commands/explain.rs`. Inside `print_rule`, after the `message:` loop (around line 48) and before the `match &rule.kind` block (around line 50), insert:

```rust
    if let Some(desc) = &rule.description {
        println!();
        println!("description:");
        for line in wrap_for_terminal(desc) {
            println!("  {line}");
        }
    }
```

Then add a helper at the bottom of the file:

```rust
fn wrap_for_terminal(text: &str) -> Vec<String> {
    const WRAP_WIDTH: usize = 100;
    let mut out = Vec::new();
    for line in text.lines() {
        if line.is_empty() {
            out.push(String::new());
            continue;
        }
        let mut current = String::new();
        for word in line.split_whitespace() {
            if current.is_empty() {
                current.push_str(word);
            } else if current.len() + 1 + word.len() <= WRAP_WIDTH {
                current.push(' ');
                current.push_str(word);
            } else {
                out.push(std::mem::take(&mut current));
                current.push_str(word);
            }
        }
        if !current.is_empty() {
            out.push(current);
        }
    }
    out
}
```

(Yes, this duplicates `wrap_description` from `rules.rs`. We could hoist it to a shared util module, but for two call sites YAGNI — leave it duplicated. Revisit when a third caller appears.)

- [ ] **Step 4: Run**

Run: `cargo test -p lintropy-cli --test cli_explain_describe`

Expected: both pass.

Also run the existing explain tests:

Run: `cargo test -p lintropy-cli --test cli_explain`

Expected: all still pass.

- [ ] **Step 5: Commit**

```bash
git add \
  crates/lintropy-cli/src/commands/explain.rs \
  crates/lintropy-cli/tests/cli_explain_describe.rs
git commit -m "feat(cli): lintropy explain shows description block

The description block is emitted between message and query when the
rule defines a description. Wrapping at 100 columns, 2-space indent.
"
```

---

## Task 9: Backfill `description` on the three bundled example rules

**Files:**
- Modify: `.lintropy/no-unwrap.rule.yaml`
- Modify: `.lintropy/no-todo.rule.yaml`
- Modify: `.lintropy/no-dbg.rule.yaml`

- [ ] **Step 1: Update `no-unwrap.rule.yaml`**

Replace the file contents with:

```yaml
severity: warning
description: |
  Flags `.unwrap()` on Result/Option outside macro bodies. Unwraps
  panic in production paths; prefer `?`, `.expect("<context>")`, or
  an explicit `match`.
message: "avoid .unwrap() on `{{recv}}`"
fix: '{{recv}}.expect("TODO: handle error")'
language: rust
query: |
  (call_expression
    function: (field_expression
      value: (_) @recv
      field: (field_identifier) @method)
    arguments: (arguments)
    (#eq? @method "unwrap")
    (#not-has-ancestor? @method "macro_invocation")) @match
```

- [ ] **Step 2: Update `no-todo.rule.yaml`**

Replace contents with:

```yaml
severity: info
description: |
  Highlights `TODO` comments so they get filed as tracker tickets
  before merge. Untracked TODOs rot in the codebase.
message: "TODO comment — track in issue tracker before merging"
language: rust
query: |
  ((line_comment) @match
    (#match? @match "TODO"))
```

- [ ] **Step 3: Update `no-dbg.rule.yaml`**

Replace contents with:

```yaml
severity: error
description: |
  Flags stray `dbg!()` macros. They leak into logs and slow hot
  paths; delete or convert to proper tracing before merging.
message: "stray dbg! — remove before merging"
language: rust
query: |
  (macro_invocation
    macro: (identifier) @n
    (#eq? @n "dbg")) @match
```

- [ ] **Step 4: Verify examples still load and dogfood pass**

Run: `cargo run -p lintropy-cli -- config validate`

Expected: exit 0, message like `OK — <N> rules loaded`.

Run: `cargo run -p lintropy-cli -- rules`

Expected: multi-line output with descriptions visible under the three rule ids. Visual inspection only.

Run the whole test suite to make sure no fixture-sensitive assertion broke:

```bash
cargo test --workspace
```

Expected: all green.

- [ ] **Step 5: Commit**

```bash
git add .lintropy/no-unwrap.rule.yaml .lintropy/no-todo.rule.yaml .lintropy/no-dbg.rule.yaml
git commit -m "chore(examples): backfill description on bundled rules"
```

---

## Task 10: Final workspace check

**Files:** none

- [ ] **Step 1: Run the full workspace test suite**

Run: `cargo test --workspace`

Expected: all green.

- [ ] **Step 2: Check formatting and lints**

Run: `cargo fmt --all -- --check`

Expected: clean.

Run: `cargo clippy --workspace --all-targets -- -D warnings`

Expected: clean. If clippy complains about new code, fix inline and
amend the relevant commit (or add a follow-up commit).

- [ ] **Step 3: Smoke-test the CLI by hand**

```bash
cargo run -p lintropy-cli -- rules
cargo run -p lintropy-cli -- rules --group-by language
cargo run -p lintropy-cli -- rules --group-by tag
cargo run -p lintropy-cli -- rules --format json | head -40
cargo run -p lintropy-cli -- explain no-unwrap
```

Expected: output matches the spec. Fix any surprises before declaring done.

- [ ] **Step 4: (If anything needed touching) commit the fixes**

Only if clippy / fmt / smoke tests surfaced issues:

```bash
git add -p
git commit -m "fix: post-review cleanup"
```

---

## Self-Review Notes

- **Spec coverage:** every §3–§9 item maps to a task. §3 → T1. §4.1/4.2/4.3 → T3/T5/T6/T7. §5 (JSON) → T3 (prod) + T4 (test). §6 (explain) → T8. §7 (backwards compat) — verified via T3 step 7 (existing `cli_rules.rs` assertions still hold). §8 (tests) → T1 + T3–T8. §9 (example backfill) → T9. §9.1 (SKILL.md) — already landed in commit `2a17018`.
- **Placeholder scan:** no TBDs, no "implement later", every code step has concrete code.
- **Type consistency:** `GroupBy` enum is defined in T2 and used by the renderer in T3 under the same path (`crate::cli::GroupBy`). `RuleConfig.description` is added in T1 step 5 and consumed in T3 / T8.
- **Known duplication:** `wrap_description`/`wrap_for_terminal` duplicated across `rules.rs` and `explain.rs`. Call sites = 2, hoisting noted as YAGNI in T8 step 3.
