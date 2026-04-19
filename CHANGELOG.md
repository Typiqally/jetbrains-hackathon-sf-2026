# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] — unreleased

First release of the Phase 1 MVP (tracked in `specs/merged/2026-04-18-lintropy-merged.md`, §13.1).

### Added

- **Config surface.** `lintropy.yaml` root config with `settings.fail_on` /
  `settings.default_severity`; rule discovery from `.lintropy/**/*.{rule,rules}.yaml`;
  JSON schema emission via `lintropy schema`.
- **Query and match rules.** Tree-sitter `query:` rules with
  `@match`/`@capture` conventions and `{{capture}}` message/fix interpolation;
  regex `match:` rules for plain-text conventions.
- **Custom predicates.** `#has-ancestor?`, `#not-has-ancestor?`,
  `#has-child?`, `#not-has-child?`, `#not-has-preceding-comment?`, and the
  file-scope `#in-file?` / `#not-in-file?` pair.
- **Engine.** Parallel per-file execution via `rayon`, predicate filtering,
  deterministic diagnostic ordering.
- **Reporters.** Rustc-style text reporter (with rule-source and `explain`
  hint) + canonical JSON envelope (§7.3).
- **Autofix.** In-place `--fix` with overlap detection; unified-diff
  `--fix-dry-run`; atomic writes.
- **Suppressions.** `// lintropy-disable-next-line[(ids)]`,
  file-level `// lintropy-disable(ids)`, and an
  unused-directive reporter that surfaces never-matched IDs.
- **CLI.** `check` (default), `explain`, `rules`, `init`, `init --with-skill`,
  `schema`, `config validate`, `ts-parse`, and `hook` subcommands per §9.
- **Agent integration.** `init --with-skill` installs the canonical
  `SKILL.md` into `.claude/skills/lintropy/` and `.cursor/skills/lintropy/`
  and merges a `PostToolUse` entry into `.claude/settings.json`
  idempotently. `lintropy hook` reads a Claude-style stdin payload, scopes
  the engine to one file, and writes diagnostics to stderr with exit code
  `0` / `2` per §15.
- **Example repos.** `examples/rust-demo/` with four canonical diagnostics
  across three files, used as the integration-test fixture. Parallel
  single-language demos ship for the other supported languages:
  `examples/go-demo/` (`no-fmt-println`, `no-todo-comment`),
  `examples/python-demo/` (`no-print`, `no-todo-comment`), and
  `examples/typescript-demo/` (`no-console-log`, `no-any-type`,
  `no-todo-comment`; covers both `.ts` and `.tsx`). Each ships with a
  `README.md` listing the expected diagnostics for
  `lintropy check examples/<lang>-demo --config examples/<lang>-demo/lintropy.yaml`.
- **Integration tests.** `tests/integration_{check,fix,hook,init}.rs` at
  the crate root exercise the full pipeline end-to-end.
- **CI.** fmt + clippy (`-D warnings`) + test matrix
  (`ubuntu-latest` + `macos-latest`) + non-blocking `cargo deny` on stable
  Rust 1.95.
- **Languages.** First-class Go, Python, and TypeScript support in
  addition to Rust. Enabled by default via Cargo features `lang-go`,
  `lang-python`, `lang-typescript`; build a Rust-only binary with
  `cargo install lintropy --no-default-features`. TypeScript covers
  `.ts`, `.tsx`, `.mts`, `.cts`, and `.d.ts`; the CLI picks the
  `typescript` vs `tsx` grammar per file. Rules declare
  `language: typescript` for both.
- **`ts-parse` auto-detect.** `lintropy ts-parse <file>` now derives the
  language from the file extension by default; `--lang <name>` remains
  as an explicit override. Error messages list every compiled-in
  language so the user sees exactly what is available.
- **Language server.** `lintropy lsp` subcommand runs a `tower-lsp`-backed
  Language Server Protocol server over stdio. Publishes diagnostics on
  `didOpen`/`didChange`/`didSave`, applies `TextDocumentSyncKind::INCREMENTAL`
  range edits in place, exposes autofixes as `CodeAction` + `WorkspaceEdit`
  quickfixes, and reloads the rule set on `.lintropy/**/*.yaml` changes via
  `didChangeWatchedFiles`.
- **VS Code / Cursor extension.** `editors/vscode/lintropy/` packages a
  `vscode-languageclient`-based extension that spawns `lintropy lsp` and
  surfaces diagnostics, quickfixes, and config reload inside the editor.
  Settings: `lintropy.enable`, `lintropy.path`, `lintropy.trace.server`.
  Release workflow publishes `lintropy-<version>.vsix` as a GitHub release
  asset alongside the CLI tarballs.
- **JetBrains integration.** `editors/jetbrains/README.md` documents the
  LSP4IJ-based setup that works on all JetBrains IDEs including free
  Community editions.
- **Init scaffolding.** `lintropy init` now creates
  `.vscode/extensions.json` recommending `lintropy.lintropy` +
  `redhat.vscode-yaml`. Skipped when the file already exists.
- **One-command editor install.** `lintropy install-lsp-extension vscode|cursor`
  downloads the matching `.vsix` from the GitHub release and hands it to
  `code`/`cursor --install-extension`. `lintropy install-lsp-template jetbrains`
  unpacks the embedded LSP4IJ custom template so users can import it with
  pre-filled fields (name, command, `*.rs → rust` mapping).
- **Auto-download binary.** The VS Code / Cursor extension resolves the
  `lintropy` binary via: `lintropy.path` → PATH → GitHub Release download
  into the extension's global storage. New `lintropy.binarySource` setting
  (`auto` by default) controls the auto-download fallback.
- **LSP4IJ template committed.** `editors/jetbrains/lsp4ij-template/` ships
  the end-user template; `editors/jetbrains/lsp4ij-template-dev/` ships the
  `$PROJECT_DIR$/target/debug/lintropy` variant for contributors iterating
  on the server.
- **Unified editor install.** `lintropy install-editor vscode|cursor|jetbrains`
  replaces the previous multi-command dance: one call installs everything
  that makes sense for that editor. The separate `lintropy-query-syntax`
  extension is merged into the main `lintropy` VS Code / Cursor extension
  (one `.vsix` contributes both the LSP client and the `query: |` grammar
  injection). `install-query-extension` is removed; use
  `install-editor vscode` or `install-lsp-extension vscode`. For JetBrains,
  `install-editor jetbrains` unpacks the LSP4IJ template into `--dir`.
- **Server-side query DSL highlighting.** `lintropy lsp` now advertises
  the `textDocument/semanticTokens` capability and tokenises the
  tree-sitter `query: |` DSL inside `lintropy.yaml` / `.lintropy/**/*.yaml`
  — captures (`decorator` + `definition`), predicates (`macro`), node
  kinds (`class`), field names (`property`), the `_` wildcard
  (`keyword`), parens/brackets/quantifiers (`operator`), strings,
  numbers, comments. Nine-type legend picked so every default editor
  scheme paints the DSL distinctly with zero per-user configuration.
  Renders in VS Code / Cursor / Neovim / Helix / Zed; JetBrains LSP4IJ
  currently discards the overlay because its highlighter only paints
  PSI leaf elements and the YAML plugin represents `query: |` bodies
  as composite `YAMLBlockScalarImpl` — fix requires a dedicated plugin
  overriding `LSPSemanticTokensFeature`, not shipped yet. The standalone
  TextMate bundle and `install-textmate-bundle` subcommand are removed
  — the server is now the single source of truth for query syntax
  colouring.
- **build.rs removed.** The previous build script existed only to pack the
  syntax-only `.vsix`; now that the merged extension is built via
  `vsce package` and released as a GitHub asset, the build-time zip is
  unnecessary (and the `zip` + build-dep `serde_json` drop off).
- **Inline linting of rule files.** When an open buffer matches
  `lintropy.yaml` / `.lintropy/**/*.{rule,rules}.yaml`, the LSP server
  validates each rule's tree-sitter query, reports compile errors
  anchored to the line (and best-effort column) inside the `query: |`
  block, flags unknown or missing `language:`, and warns on
  `{{capture}}` references in `message` / `fix` that don't exist in
  the query. No waiting for `lintropy check` — red squigglies surface
  as you type.

### Changed

- **Unified install command.** Every editor install is now a subcommand of
  `lsp`: `lintropy lsp install vscode|cursor|jetbrains|claude-code`. The
  old `install-editor`, `install-lsp-extension`, `install-lsp-template`,
  and `install-claude-code-plugin` commands are removed. VS Code / Cursor
  flags (`--package-only`, `-o`, `--profile`), JetBrains flags (`--dir`,
  `--force`), and Claude Code flags (`--scope`, `--no-install`) now live
  on the one subcommand.
- **Claude Code integration.**
  - New `.claude-plugin/marketplace.json` at the repo root lets users run
    `/plugin marketplace add Typiqally/lintropy` then `/plugin install
    lintropy-lsp@lintropy` without any local CLI.
  - `lintropy lsp install claude-code` generates the plugin manifest
    freshly — version pinned to `CARGO_PKG_VERSION`, `extensionToLanguage`
    feature-gated, `command` resolved to an absolute `lintropy` binary
    path — and shells out to `claude plugin install <dir> --scope
    <scope>` when the `claude` CLI is on `PATH`. `--no-install` skips the
    shell-out.
- **Internal API.** `Language::ts_language` now takes a `&Path` argument
  so TypeScript can dispatch between the `typescript` and `tsx`
  grammars. Other languages ignore it. Not a published SDK surface.

### Notes

- The default binary grows by roughly 5–7 MB because it bundles the
  three additional tree-sitter grammars. `--no-default-features`
  produces a Rust-only build of the same size as before.

[0.1.0]: https://github.com/Typiqally/lintropy/releases/tag/v0.1.0
