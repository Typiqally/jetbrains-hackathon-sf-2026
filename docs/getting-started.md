# Getting Started

This guide is the fastest path from zero to a working repo-local lint.

## 1. Install

Homebrew:

```console
brew tap Typiqally/lintropy
brew install lintropy
```

From source:

```console
cargo install --path .
```

## 2. Scaffold a repo

Inside the repository you want to lint:

```console
lintropy init
```

That creates:

- `lintropy.yaml`
- `.lintropy/no-unwrap.rule.yaml`
- `.vscode/extensions.json` if it does not already exist

If you also want the bundled agent skill installed into `.claude/` or `.cursor/`:

```console
lintropy init --with-skill
```

## 3. Inspect the generated files

Minimal root config:

```yaml
version: 1
settings:
  fail_on: error
  default_severity: warning
```

Starter rule:

```yaml
language: rust
severity: warning
message: "avoid .unwrap() on `{{recv}}` — handle the error explicitly"
include: ["**/*.rs"]
query: |
  (call_expression
    function: (field_expression
      value: (_) @recv
      field: (field_identifier) @method)
    (#eq? @method "unwrap")) @match
fix: '{{recv}}.expect("TODO: handle error")'
```

## 4. Validate config

Before scanning the repo, make sure the config loads:

```console
lintropy config validate
```

Expected shape:

```text
OK: N rules loaded from /path/to/repo
```

## 5. Run a check

Run against the current directory:

```console
lintropy check .
```

Useful variants:

```console
lintropy check . --format json
lintropy check . --quiet
lintropy check . --config ./lintropy.yaml
```

## 6. Apply or preview fixes

Preview autofixes as a unified diff:

```console
lintropy check . --fix-dry-run
```

Apply autofixes in place:

```console
lintropy check . --fix
```

Only rules with a `fix:` template produce fixes.

## 7. Inspect loaded rules

List rules:

```console
lintropy rules
```

Explain one rule:

```console
lintropy explain no-unwrap
```

This is the fastest way to confirm which file defined the rule, what message it emits, and what query it is actually running.

## 8. Write your next rule

For structural rules, do this in order:

1. Pick a real example file.
2. Dump its Tree-sitter shape with `lintropy ts-parse path/to/file.rs`.
3. Write a query rule under `.lintropy/`.
4. Run `lintropy config validate`.
5. Run `lintropy check .`.

Use [Rule Language](./rule-language.md) for the exact syntax split between Tree-sitter and Lintropy.

## Common first rules

Good first rules:

- ban `dbg!`
- ban `println!` outside tests
- ban `.unwrap()`
- require a safety comment before `unsafe`
- restrict certain imports to one directory

## What to expect today

Today the practical happy path is:

- Rust repo
- query rules
- optional autofix

If you try to write `forbid:` or `require:` rules, config loading will fail because that rule kind is not active yet.
