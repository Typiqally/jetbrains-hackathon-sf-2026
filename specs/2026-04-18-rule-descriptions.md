# Rule descriptions and richer `rules` listing

- Date: 2026-04-18
- Status: Approved design, ready for implementation
- Scope: Additive. No breaking changes to existing rule YAML, CLI flags, or JSON output contracts.

## 1. Motivation

Each rule today carries a `message` (the diagnostic text shown when the rule
fires, templated with `{{capture}}` substitutions) plus metadata (`tags`,
`docs_url`, `severity`, etc.). There is no place to record *why* the rule
exists — the intent behind the pattern, the class of bug it catches, or the
recommended remediation in prose form. Authors work around this by cramming
rationale into `message`, which then leaks into every diagnostic and cannot be
formatted for long-form reading.

The `lintropy rules` subcommand lists every loaded rule, but its output
currently shows only `id`, `severity`, and source path. That is enough for
"is the rule loaded?" checks, but not enough to explain the rule catalogue to
a human reading it for the first time — a key onboarding surface and a
prerequisite for the SKILL.md story (WP6) where agents need to discover
available rules.

This spec adds:

1. An optional `description` field on every rule — free-form prose, never
   substituted into diagnostics, intended for human and agent consumption.
2. Richer default output from `lintropy rules`, plus a `--group-by` flag for
   organising larger catalogues.
3. Propagation of `description` through `lintropy explain <id>` and
   `lintropy rules --format json`.

## 2. Non-goals

- Markdown rendering of the description. The field is plain text. Terminals
  render it verbatim.
- Length limits or a linter-on-linter that validates description style.
- Localisation. Single-language field.
- Nested documentation blocks (`docs: { summary, rationale, example }`). The
  door is left open for a future additive extension, but this spec ships the
  single flat field only.
- Changes to the diagnostic output of `lintropy check`. Descriptions never
  surface in per-violation output — that is what `message` is for.

## 3. Schema change

### 3.1 `RawRule` (`crates/lintropy-core/src/config.rs`)

Add one field:

```rust
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct RawRule {
    // ... existing fields ...
    #[serde(default)]
    description: Option<String>,
    // ... existing fields ...
}
```

The `deny_unknown_fields` attribute already guards against typos; adding the
field to the deserialiser is the only mechanical change.

### 3.2 `RuleConfig` (`crates/lintropy-core/src/config.rs`)

Add one field to the resolved struct:

```rust
pub struct RuleConfig {
    // ... existing fields ...
    pub description: Option<String>,
    // ... existing fields ...
}
```

`build_rule` copies `raw.description` into `RuleConfig.description`
verbatim — no trimming, no template scanning, no normalisation of newlines.
Authors control whitespace. An explicit empty string is normalised to
`None` at build time so downstream code only needs to check for `Some`.

### 3.3 JSON Schema

The `schemars` derive on `RawRule` picks up the new field automatically.
`lintropy schema` therefore exposes `description` with no further code change.
A test (`json_schema_exposes_description`) asserts the property is present
in the generated schema.

### 3.4 Supported locations

`description` is valid in every existing rule YAML format:

- Inline `rules:` block inside `lintropy.yaml`.
- Single-rule files: `.lintropy/<stem>.rule.yaml`.
- Bundle files: `.lintropy/<name>.rules.yaml` (each entry under `rules:`).

No new discovery logic.

### 3.5 Not templated

Unlike `message` and `fix`, the `description` field is **not** scanned for
`{{capture}}` tokens. Authors may mention captures by name in prose, but the
loader does not attempt substitution. This keeps the field safe for long-form
content that happens to contain braces.

## 4. `lintropy rules` text output

### 4.1 Default (flat, sorted by id)

Each rule renders as a header line followed by an indented body:

```
no-unwrap  [warning]  rust
  avoid .unwrap() on Result/Option; prefer ? or .expect() with context
  tags: safety, rust
  source: .lintropy/no-unwrap.rule.yaml

no-todo    [error]    rust
  flag stray TODO comments so tracker tickets get filed
  source: .lintropy/no-todo.rule.yaml
```

Formatting rules:

- Header line: `<id>  [<severity>]  <language>`.
  - `id` column is left-aligned and padded to the longest id across all rules
    in the current group (or across all rules when `--group-by none`).
  - `[<severity>]` column is left-aligned and padded to the longest severity
    label within the same scope, so the language column aligns.
  - Two-space gaps between columns.
  - `language` column shows `<lang-name>` or is blank when `language == None`.
- Body lines are indented with two spaces:
  - `description` text, soft-wrapped on word boundaries at 100 columns. Each
    wrapped continuation line is re-indented with two spaces. Missing
    description → skip the description line(s) entirely. An explicit empty
    string (`description: ""`) is treated the same as absent.
  - `tags: <comma-separated list>` when `tags` is non-empty.
  - `docs: <url>` when `docs_url.is_some()`.
  - `source: <path>` always.
- A single blank line separates rules.

Rules are sorted by `id` ascending.

### 4.2 `--group-by <MODE>` flag

Added to `RulesArgs` in `crates/lintropy-cli/src/cli.rs`. Accepts one of:

- `none` (default): flat output as in §4.1.
- `language`: group by `language.name()`, ascending. Rules with `None`
  language go into a bucket labelled `(any)`, printed last.
- `tag`: group by the **first** tag in each rule's declared tag list. A rule
  appears under one group only (the first-tag bucket); this avoids
  duplication and keeps output stable. Rules with no tags go into a bucket
  labelled `(untagged)`, printed last.

Within every group, rules are sorted by `id` ascending.

Group output format:

```
rust
----
<rules in §4.1 format>

python
------
<rules>

(any)
-----
<rules>
```

The underline length matches the group name.

### 4.3 Interaction with `--format`

`--group-by` applies to text format only. When the user passes both
`--group-by <anything other than none>` and `--format json`, the command
exits with a user error: `--group-by only applies to text format`. JSON
consumers group on their own.

## 5. `lintropy rules --format json`

Each rule object gains one key:

```json
{
  "id": "no-unwrap",
  "severity": "warning",
  "language": "rust",
  "kind": "query",
  "description": "avoid .unwrap() on Result/Option; prefer ? or .expect() with context",
  "source_path": ".lintropy/no-unwrap.rule.yaml",
  "tags": ["safety", "rust"],
  "docs_url": null,
  "include": [],
  "exclude": []
}
```

`description` is always present, with value `null` when the rule does not
supply one. The top-level shape stays a flat array, ordered by `id`
ascending. `--group-by` has no effect on JSON output (see §4.3).

## 6. `lintropy explain <id>` output

When `description.is_some()`, insert a new block **after** the `message:`
block and **before** the `query:` / `forbid:` / `require:` block:

```
description:
  <description text, wrapped at 100 cols, 2-space indent>
```

The description text is printed line-for-line from the author-supplied value,
with wrapping applied at 100 columns and a two-space indent on every line
(wrapped continuations included). When `description.is_none()`, the
`description:` block is omitted entirely.

No other parts of `explain` change.

## 7. Backwards compatibility

- Existing rule YAML loads unchanged: `description` is `#[serde(default)]`.
- JSON output for `rules --format json` gains one key; existing consumers
  that ignore unknown keys keep working. Consumers that asserted an exact
  key set will need to accept `description`. No public JSON contract
  exists outside this repo today.
- `rules` text output changes. Any script that grepped the old single-line
  format will need to be rewritten. The old format was never guaranteed
  stable; this spec establishes the new one as the documented format.

## 8. Testing

### 8.1 `lintropy-core`

Unit tests in `crates/lintropy-core/src/config.rs`:

- `description_roundtrip` — YAML with `description: "..."` round-trips to
  `RuleConfig.description == Some("...")`.
- `description_absent_is_none` — missing field resolves to `None`.
- `description_multiline_preserved` — a block scalar (`description: |\n  line1\n  line2\n`)
  preserves newlines verbatim in the resolved value.
- `json_schema_exposes_description` — `Config::json_schema()` includes
  `description` under `properties`.

### 8.2 `lintropy-cli`

Integration tests using `assert_cmd` plus a new fixture directory
`crates/lintropy-cli/tests/fixtures/describe/` containing four rules:

- one described + tagged + Rust,
- one described + untagged + Python,
- one undescribed + tagged + Rust,
- one undescribed + untagged + no language.

Tests:

- `rules_text_default_shows_description_line`
- `rules_text_hides_description_when_absent`
- `rules_text_group_by_language_groups_and_sorts`
- `rules_text_group_by_tag_first_tag_wins`
- `rules_text_group_by_language_buckets_no_language_as_any`
- `rules_text_group_by_tag_buckets_untagged_last`
- `rules_json_includes_description_null_when_absent`
- `rules_json_includes_description_string_when_present`
- `rules_rejects_group_by_with_json_format`
- `explain_prints_description_block_when_present`
- `explain_omits_description_block_when_absent`

Snapshot strategy: use `insta` if already in the workspace (check during
implementation); otherwise plain string assertions on stdout. Either is
acceptable; the implementer picks based on existing crate conventions.

## 9. Example rule updates

The three bundled example rules at `.lintropy/*.rule.yaml` gain descriptions
to demonstrate the field and to keep the repo's dogfood fixture realistic:

- `no-unwrap.rule.yaml`: "Flags `.unwrap()` on Result/Option outside macro
  bodies. Unwraps panic in production paths; prefer `?`, `.expect(\"<context>\")`,
  or explicit match."
- `no-todo.rule.yaml`: short blurb on tracker hygiene.
- `no-dbg.rule.yaml`: short blurb on debug-macro leaks in shipped code.

These are illustrative only; wording is at the implementer's discretion.

## 10. File-level summary of changes

| File | Change |
|------|--------|
| `crates/lintropy-core/src/config.rs` | Add `description` to `RawRule` and `RuleConfig`; wire in `build_rule`; new unit tests. |
| `crates/lintropy-cli/src/cli.rs` | Add `--group-by <MODE>` to `RulesArgs`; enum `GroupBy { None, Language, Tag }`. |
| `crates/lintropy-cli/src/commands/rules.rs` | New multi-line text renderer; grouping logic; JSON serialiser adds `description`; error on `--group-by + --format json`. |
| `crates/lintropy-cli/src/commands/explain.rs` | Insert `description:` block when present. |
| `crates/lintropy-cli/tests/fixtures/describe/` | New fixture directory with four rules. |
| `crates/lintropy-cli/tests/rules_describe.rs` | New test file covering §8.2. |
| `crates/lintropy-cli/tests/explain_describe.rs` | New test file covering `explain` cases. |
| `.lintropy/no-unwrap.rule.yaml`, `no-todo.rule.yaml`, `no-dbg.rule.yaml` | Backfill `description`. |

## 11. Out-of-scope follow-ups

Recorded for future design work, not to be done here:

- Nested `docs` block with `summary`, `rationale`, `example` subfields.
- `lintropy check`-time emission of description into SARIF `help.text`
  fields.
- Agent-friendly `rules --format jsonl` for streaming large catalogues.
- Fuzzy search: `lintropy rules --search <term>` matching over id + tags +
  description.
