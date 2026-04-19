# Editor And Agent Setup

Lintropy ships a single LSP server (`lintropy lsp`) and one install command that wires it into every supported editor or agent:

```console
lintropy lsp install <target>
```

`<target>` is one of `vscode`, `cursor`, `jetbrains`, `claude-code`. The rest of this page walks through each target plus a couple of helpers that live beside it.

## VS Code and Cursor

```console
lintropy lsp install vscode
lintropy lsp install cursor
```

This builds the bundled extension source into a `.vsix` and hands it to `code --install-extension` / `cursor --install-extension`. The extension carries:

- the LSP client (diagnostics, quickfixes, config reload)
- semantic-token highlighting for the `query: |` DSL inside rule files

No separate "query syntax" extension.

### Flags

- `--profile <NAME>` — install into a named editor profile.
- `--package-only -o <PATH>` — build the `.vsix` but do not run the editor CLI. Useful in CI.

### Binary resolution

The installed extension resolves `lintropy` in this order: explicit `lintropy.path` setting → `PATH` lookup → extension-managed download from the matching GitHub release.

### Config resolution

Config is resolved path-locally, not workspace-wide:

- each source file uses the nearest ancestor `lintropy.yaml`
- a newly added nested `lintropy.yaml` creates a fresh rule context for that subtree and does not inherit the parent workspace rules
- changes under that root's `.lintropy/` directory merge into the same context
- saving or otherwise notifying the editor about `lintropy.yaml` / `.lintropy/**/*.yaml` changes triggers a config reload and republishes diagnostics for open files

## JetBrains IDEs

```console
lintropy lsp install jetbrains --dir ~/.lintropy
```

This unpacks the [LSP4IJ](https://plugins.jetbrains.com/plugin/23257-lsp4ij) custom server template. One import step in the IDE:

`View → Tool Windows → LSP Console → + → New Language Server → Template → Import from directory…`

Pick the extracted directory (default name `lsp4ij-template`). All fields — name, command, `*.rs → rust`, `*.rule.yaml → yaml` mappings — are pre-filled.

### Flags

- `--dir <PATH>` — parent directory for the extracted template.
- `--force` — overwrite an existing template directory.

## Claude Code

Lintropy ships a Claude Code plugin that registers `lintropy lsp` as a Language Server. Two paths to install it; pick whichever matches your setup.

### Marketplace (recommended)

Inside Claude Code:

```text
/plugin marketplace add Typiqally/lintropy
/plugin install lintropy-lsp@lintropy
```

The marketplace manifest lives at the root of the GitHub repo, so this works from any clean Claude Code install — no `lintropy` CLI needed to bootstrap.

For a local checkout, point the marketplace at the absolute path instead:

```text
/plugin marketplace add /absolute/path/to/lintropy
/plugin install lintropy-lsp@lintropy
```

This reads the same `editors/claude-code/.claude-plugin/plugin.json` straight from disk, so edits to the manifest take effect after `/plugin marketplace update lintropy`.

### CLI

```console
lintropy lsp install claude-code
```

Generates the plugin manifest fresh (version synced to the installed `lintropy`, extension map scoped to the compiled-in languages, `command` resolved to the absolute binary path) and then shells out to `claude plugin install <dir> --scope <scope>` when the `claude` CLI is on `PATH`.

#### Flags

- `--scope project` (default) — team-shared, recorded in `.claude/settings.json`.
- `--scope user` — personal-only install.
- `--no-install` — write the plugin directory but do not shell out; prints the `claude plugin install` command for you to run.
- `--dir <PATH>` — write the plugin directory somewhere other than the cwd.
- `--force` — overwrite an existing plugin directory.

Prefer the CLI path when you want the generated `command` to be an absolute path to the local `lintropy` binary, or when your Claude Code subprocess environment has a different `PATH` than your shell.

### What the plugin does

It maps YAML, Rust, Go, Python, and TypeScript file extensions to the LSP language ids `lintropy lsp` expects. The extension map is feature-gated, so a binary built without a language feature won't register `lintropy` for that file type. No environment variable is needed — Claude Code's LSP tool activates automatically once a plugin registers a server.

## Other LSP-capable editors

Any editor that can launch an LSP server over stdio can use:

```console
lintropy lsp
```

The server:

- resolves the nearest ancestor `lintropy.yaml` per file instead of using one workspace-wide config
- treats a nested `lintropy.yaml` as a fresh rule context for that subtree
- merges `.lintropy/` rule files into the resolved context for that root
- republishes diagnostics when watched config files change
- supports quickfix code actions for diagnostics with `fix`

## Other agents

| Agent | LSP support | Notes |
| --- | --- | --- |
| Continue | Partial | Wrap `lintropy lsp` behind an MCP bridge and connect it through Continue's MCP config. |
| Cursor (agent mode) | IDE only | The Cursor IDE already runs the LSP extension (see above); the in-IDE agent sees those diagnostics without extra setup. |
| Aider | No | No LSP client in the CLI. Use the post-write hook instead. |
| Codex CLI | No | No maintained LSP client. Use the post-write hook instead. |

For any agent not listed here: if it launches an LSP server over stdio, the command is always `lintropy lsp` with no arguments. If it only supports hook-style integration, use the post-write hook below.

## JSON Schema support

Lintropy ships JSON Schemas for:

- `lintropy.yaml`
- `.lintropy/*.rule.yaml`
- `.lintropy/*.rules.yaml`

YAML-aware editors use these for completion, hover docs, and validation. VS Code / Cursor pick them up through `.vscode/settings.json`. JetBrains IDEs pick them up through `.idea/jsonSchemas.xml`.

## Post-write hook

### `lintropy init --with-skill`

When `.claude/` or `.cursor/` exists, this command installs the bundled `SKILL.md` into the appropriate skill directory.

For Claude Code, it also updates `.claude/settings.json` to add a `PostToolUse` command hook:

```text
lintropy hook --agent claude-code
```

### `lintropy hook`

This command is designed for machine-to-machine use, not direct human use.

Behavior:

- reads JSON payloads from stdin
- extracts a written file path
- skips work if the file is gitignored
- lints only that file
- emits compact text or JSON diagnostics to stderr
- returns a blocking exit code when diagnostics meet the configured hook threshold

Current agent support:

- Claude Code hook payloads are the implemented target
- Codex is present as a CLI option, but auto-detection is still effectively Claude-first

## Recommended setup

For most teams:

1. Run `lintropy init --with-skill`.
2. Run `lintropy lsp install <target>` for your editor and, if you use Claude Code, for `claude-code`.
3. Keep `lintropy check .` in CI.
4. Use the hook only for fast feedback after edits, not as the only enforcement point.
