# Troubleshooting

This page covers the most common user-facing failure modes.

## `lintropy check` exits with code `2`

That means a user/config error, not a lint finding.

Common causes:

- invalid YAML
- duplicate rule ids
- unknown `language`
- invalid Tree-sitter query
- unknown custom predicate
- `message` or `fix` references a capture that the query does not define
- trying to use `forbid` or `require` rules before they are supported

Start here:

```console
lintropy config validate
```

## My rule loads, but the highlight span is wrong

The rule probably omitted `@match`.

Lintropy uses `@match` as the diagnostic span and autofix replacement range. Without it, the engine falls back to the first capture, which is often too broad.

## My fix did not apply

Check these cases:

- the rule has no `fix:` field
- the query did not produce an `@match` capture where you expected
- the fix overlapped another fix in the same file
- you ran `--fix-dry-run` instead of `--fix`

Preview first:

```console
lintropy check . --fix-dry-run
```

## `language: rust` works, other languages do not

That is the current product state. The shipped language registry only supports Rust today.

## `forbid:` / `require:` looks documented but fails

Those fields exist in the config shape, but match-rule execution is not enabled yet. The loader currently rejects them as Phase 2.

## Query highlighting is missing in my editor

Query highlighting ships inside the main LSP extension. If it is missing, the
LSP integration itself is not running. Install or reinstall it:

```console
lintropy lsp install vscode       # or: cursor, jetbrains
```

JetBrains note: semantic tokens for the `query: |` DSL are not painted inside
JetBrains IDEs (LSP4IJ discards them for composite PSI elements). Diagnostics
and inline rule-file linting still work. See [`editors/jetbrains/README.md`](../editors/jetbrains/README.md).

## Live diagnostics are missing in my editor

Check:

- the LSP extension or template is installed
- the `lintropy` binary is on `PATH`, or the extension is configured to find it
- the file you opened has a valid ancestor `lintropy.yaml`
- the file extension is one of the supported language extensions

The LSP resolves config per file:

- a nested `lintropy.yaml` replaces the parent rule context for files under that subtree
- `.lintropy/` files merge into the context owned by the nearest `lintropy.yaml`
- after editing or adding rule files, save them or otherwise trigger watched-file notifications so diagnostics republish

If in doubt, run this from the directory whose `lintropy.yaml` should own the file:

```console
lintropy config validate
lintropy check .
```

If those fail, the editor integration will fail too.

## Suppression comments are ignored

Current suppression support is strict:

- only `// lintropy-ignore: rule-id`
- only `// lintropy-ignore-file: rule-id`
- directives must be on their own line
- file directives must appear within the first 20 lines
- wildcards are rejected

Trailing code like this will not work:

```rust
do_work(); // lintropy-ignore: some-rule
```

## `lintropy hook` appears to do nothing

That usually means one of these:

- stdin did not contain a recognized JSON payload
- the payload had no usable file path
- the file path did not resolve
- the file was gitignored
- config failed to load

The hook command is intentionally conservative and often exits `0` rather than failing noisy in automation glue.
