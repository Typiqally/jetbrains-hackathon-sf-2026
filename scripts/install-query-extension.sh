#!/usr/bin/env bash

set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  ./scripts/install-query-extension.sh <vscode|cursor> [--profile NAME] [--package-only]

Examples:
  ./scripts/install-query-extension.sh vscode
  ./scripts/install-query-extension.sh cursor
  ./scripts/install-query-extension.sh cursor --profile "Default"
  ./scripts/install-query-extension.sh vscode --package-only
EOF
}

if [[ $# -lt 1 ]]; then
  usage
  exit 2
fi

editor=""
profile=""
package_only="false"

while [[ $# -gt 0 ]]; do
  case "$1" in
    vscode|cursor)
      if [[ -n "$editor" ]]; then
        echo "error: editor already set to '$editor'" >&2
        exit 2
      fi
      editor="$1"
      shift
      ;;
    --profile)
      if [[ $# -lt 2 ]]; then
        echo "error: --profile requires a value" >&2
        exit 2
      fi
      profile="$2"
      shift 2
      ;;
    --package-only)
      package_only="true"
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "error: unknown argument '$1'" >&2
      usage >&2
      exit 2
      ;;
  esac
done

if [[ -z "$editor" ]]; then
  echo "error: choose 'vscode' or 'cursor'" >&2
  usage >&2
  exit 2
fi

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
extension_dir="$repo_root/editors/vscode/lintropy-query-syntax"
vsix_dir="$(mktemp -d)"
vsix_path="$vsix_dir/lintropy-query-syntax.vsix"

cleanup() {
  rm -rf "$vsix_dir"
}

trap cleanup EXIT

echo "Packaging extension from $extension_dir"
(
  cd "$extension_dir"
  npx @vscode/vsce package --out "$vsix_path"
)

if [[ "$package_only" == "true" ]]; then
  cp "$vsix_path" "$repo_root/editors/vscode/lintropy-query-syntax/lintropy-query-syntax.vsix"
  echo "Packaged: $repo_root/editors/vscode/lintropy-query-syntax/lintropy-query-syntax.vsix"
  exit 0
fi

case "$editor" in
  vscode)
    cli_bin="code"
    ;;
  cursor)
    cli_bin="cursor"
    ;;
esac

if ! command -v "$cli_bin" >/dev/null 2>&1; then
  echo "error: '$cli_bin' was not found in PATH" >&2
  if [[ "$editor" == "vscode" ]]; then
    echo "Install the VS Code shell command first, then rerun this script." >&2
  else
    echo "Install the Cursor shell command first, then rerun this script." >&2
  fi
  exit 2
fi

install_cmd=("$cli_bin")
if [[ -n "$profile" ]]; then
  install_cmd+=("--profile" "$profile")
fi
install_cmd+=("--install-extension" "$vsix_path" "--force")

echo "Installing into $editor"
"${install_cmd[@]}"
echo "Installed lintropy-query-syntax into $editor"
