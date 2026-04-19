---
title: Integrations
---

# Integrations

Lintropy ships a single LSP server (`lintropy lsp`) and one install command that wires it into every supported editor or agent:

```console
lintropy install <target>
```

`<target>` is one of `vscode`, `cursor`, `jetbrains`, `claude-code`.

## Pick your integration

- [VS Code and Cursor](vscode.md) — extension `.vsix` built from source, installed via the editor CLI.
- [JetBrains IDEs](jetbrains.md) — LSP4IJ template, one-click import.
- [Claude Code](claude-code.md) — plugin + skill, loaded via `claude --plugin-dir`.
- [Other LSP editors](other-editors.md) — Neovim, Helix, Zed — anything that spawns a stdio LSP server.
- [Other agents](other-agents.md) — Continue, Aider, Codex CLI.

## Recommended setup

For most teams:

1. Run `lintropy init --with-skill`.
2. Run `lintropy install <target>` for your editor and, if you use Claude Code, for `claude-code` (that one also drops the skill).
3. Keep `lintropy check .` in CI as the enforcement point.

## JSON Schema support

Lintropy ships JSON Schemas for:

- `lintropy.yaml`
- `.lintropy/*.rule.yaml`
- `.lintropy/*.rules.yaml`

YAML-aware editors use these for completion, hover docs, and validation. VS Code / Cursor pick them up through `.vscode/settings.json`. JetBrains IDEs pick them up through `.idea/jsonSchemas.xml`.
