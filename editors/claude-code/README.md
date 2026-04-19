# Lintropy — Claude Code plugin

Registers `lintropy lsp` as a Language Server for Claude Code so lintropy
diagnostics appear live while the agent reads and edits files.

## Install

### Marketplace (recommended)

Inside Claude Code:

```text
/plugin marketplace add Typiqally/lintropy
/plugin install lintropy-lsp@lintropy
```

For a local checkout:

```text
/plugin marketplace add /absolute/path/to/lintropy
/plugin install lintropy-lsp@lintropy
```

### CLI

```console
lintropy lsp install claude-code                 # --scope project by default
lintropy lsp install claude-code --scope user    # personal-only
lintropy lsp install claude-code --no-install    # write the dir, print the install command
```

The CLI generates this plugin directory freshly (version synced to the
installed `lintropy`, extension map scoped to compiled-in languages,
`command` resolved to the absolute binary path) and shells out to
`claude plugin install` when the `claude` CLI is on `PATH`.

## What's inside

- `.claude-plugin/plugin.json` — registers the `lintropy` LSP server and
  maps the file extensions lintropy knows to their LSP language ids.

The `lintropy` binary must be on `PATH`, or override `command` in
`plugin.json` with an absolute path.
