# Lintropy — Codex plugin

Bundles the lintropy skill as a native Codex plugin so Codex can discover
repo-local lintropy workflows for rule authoring, AST inspection, and
debug loops.

## Local marketplace

This repo now includes a Codex marketplace manifest at:

```text
.agents/plugins/marketplace.json
```

That means a local checkout can be added to Codex directly:

```console
codex marketplace add /absolute/path/to/lintropy
```

The marketplace points at this plugin directory:

- `.codex-plugin/plugin.json` — Codex plugin manifest
- `skills/lintropy/SKILL.md` — bundled lintropy authoring skill

## Generated marketplace

```console
lintropy install codex
```

Generates a standalone local marketplace at `./lintropy-codex-marketplace/`
with the same plugin mirrored under `plugins/lintropy/`, ready for:

```console
codex marketplace add /absolute/path/to/lintropy-codex-marketplace
```

## Notes

This Codex integration is skill-based. Codex plugins package skills and
related surfaces such as MCP or apps; they do not currently register
`lintropy lsp` as an editor LSP client. For live diagnostics, keep using
the VS Code, Cursor, JetBrains, or other LSP editor integrations.
