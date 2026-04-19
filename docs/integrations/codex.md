---
title: Codex
---

# Codex

This repo itself is a local Codex marketplace. Add the checkout directly:

```console
codex marketplace add /absolute/path/to/lintropy
```

The marketplace manifest lives at:

```text
.agents/plugins/marketplace.json
```

and points at the bundled plugin in:

```text
editors/codex/
```

`lintropy install codex` generates the same structure as a standalone
marketplace for ad hoc local installs:

```console
lintropy install codex
codex marketplace add /absolute/path/to/lintropy-codex-marketplace
```

Default generated output directory:

```text
./lintropy-codex-marketplace/
```

Generated contents:

- `.agents/plugins/marketplace.json` — Codex marketplace manifest
- `plugins/lintropy/.codex-plugin/plugin.json` — Codex plugin manifest
- `plugins/lintropy/skills/lintropy/SKILL.md` — bundled lintropy skill
- `README.md` — quick integration notes

What this gives you:

- Installable local Codex marketplace entry for lintropy
- Native lintropy skill discovery inside Codex
- Repo-local guidance for writing and debugging `.rule.yaml` files
- A stable place to package lintropy workflows for local plugin install

What it does not give you:

- Editor-side live diagnostics through `lintropy lsp`

Codex plugins package skills and related surfaces such as MCP or apps.
They do not currently register an external LSP server the way the Claude
Code plugin does. For live diagnostics and semantic token feedback, keep
using one of the editor integrations:

- [VS Code and Cursor](vscode.md)
- [JetBrains IDEs](jetbrains.md)
- [Other LSP editors](other-editors.md)

Inside Codex itself, the expected loop is:

1. Use the bundled skill to inspect rules, author queries, and debug captures.
2. Run `lintropy ts-parse <file>` when you need the exact tree-sitter node kinds.
3. Run `lintropy check .` or a narrower path to verify the change.
