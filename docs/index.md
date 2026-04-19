# Lintropy

Repo-local linting for rules your repo actually cares about.

Put architecture boundaries, banned APIs, migration policy, and agent guardrails
in version control. One root config. Small YAML rule files. Structural matching
for Rust.

[Get Started](getting-started.md){ .md-button .md-button--primary }
[View Source](https://github.com/Typiqally/lintropy){ .md-button }

## Why teams use it

- **Repo-local:** rules live with the code they govern.
- **Structural:** Rust rules use tree-sitter queries, not brittle grep.
- **Practical:** messages, severity, and autofix stay in one place.
- **Agent-friendly:** explicit enough for humans, simple enough for coding agents.

## What it looks like

```console
$ lintropy check .

warning[no-unwrap]: avoid .unwrap() on `client`
  --> src/handlers/users.rs:42:18
  help: replace with `client.expect("TODO: handle error")`

error[api-only-in-src-api]: API handlers must live under src/api/
  --> src/features/users/create_user.rs:1:1

Summary: 1 error, 1 warning, 2 files affected.
```

## Example rule

```yaml
severity: warning
message: "avoid .unwrap() on `{{recv}}`"
fix: '{{recv}}.expect("TODO: handle error")'
language: rust
query: |
  (call_expression
    function: (field_expression
      value: (_) @recv
      field: (field_identifier) @method)
    (#eq? @method "unwrap")) @match
```

## Rules live in the repo

Review them like code. Change them with the codebase. Stop hiding project
policy in wiki pages, onboarding calls, and reviewer lore.

## Simple shape

A root `lintropy.yaml`. A `.lintropy/` folder. One rule per file, or grouped
rules where that helps.

## Good for real engineering constraints

Use it for architecture boundaries, migration rules, banned macros, dated
deprecations, naming conventions, or repo-specific review policy that generic
linters will never know.

## Start

Install with Homebrew:

```console
brew tap Typiqally/lintropy
brew install lintropy
```

Or build from source:

```console
cargo install --path .
```

Scaffold and run:

```console
lintropy init
lintropy check .
```

## Read next

- [Overview](overview.md)
- [Getting Started](getting-started.md)
- [Rule Language](rule-language.md)
- [CLI Guide](cli.md)
