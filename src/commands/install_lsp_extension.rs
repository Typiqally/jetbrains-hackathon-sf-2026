//! `lintropy install-lsp-extension vscode|cursor` — download the
//! matching-version `.vsix` from the GitHub release and hand it to the
//! editor's `--install-extension` flag.
//!
//! Pinning the downloaded extension to the binary's own version keeps
//! the LSP client and server compatible (same message shapes,
//! capabilities, quickfix schema). Users who want a specific version
//! can pass `--version` or a pre-downloaded file with `--vsix`.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

use crate::cli::{InstallLspExtensionArgs, LspExtensionEditor};
use crate::exit::{CliError, EXIT_OK};

const REPO_URL: &str = env!("CARGO_PKG_REPOSITORY");

pub fn run(args: InstallLspExtensionArgs) -> Result<u8, CliError> {
    let version = args
        .version
        .clone()
        .unwrap_or_else(|| env!("CARGO_PKG_VERSION").to_string());

    // Resolve the .vsix: either a user-supplied local path (skip network)
    // or a download from the matching GitHub release.
    let (vsix_path, owned_tmp) = match args.vsix.as_ref() {
        Some(local) => {
            if !local.is_file() {
                return Err(CliError::user(format!(
                    "--vsix path does not exist: {}",
                    local.display()
                )));
            }
            (local.clone(), None)
        }
        None => {
            let tmp = download_vsix(&version)?;
            let path = tmp.path().to_path_buf();
            (path, Some(tmp))
        }
    };

    if args.package_only {
        let default_name = format!("lintropy-{version}.vsix");
        let target = args.output.unwrap_or_else(|| PathBuf::from(&default_name));
        if let Some(parent) = target.parent().filter(|p| !p.as_os_str().is_empty()) {
            fs::create_dir_all(parent)?;
        }
        fs::copy(&vsix_path, &target)?;
        println!("packaged {}", target.display());
        drop(owned_tmp);
        return Ok(EXIT_OK);
    }

    let editor = args
        .editor
        .ok_or_else(|| CliError::user("editor is required (vscode or cursor)"))?;
    let cli_bin = match editor {
        LspExtensionEditor::Vscode => "code",
        LspExtensionEditor::Cursor => "cursor",
    };
    if which(cli_bin).is_none() {
        return Err(CliError::user(format!(
            "`{cli_bin}` not found in PATH — install the editor's shell command first"
        )));
    }

    let mut cmd = Command::new(cli_bin);
    if let Some(profile) = args.profile.as_deref() {
        cmd.arg("--profile").arg(profile);
    }
    cmd.arg("--install-extension")
        .arg(&vsix_path)
        .arg("--force");

    let status = cmd
        .status()
        .map_err(|err| CliError::internal(format!("spawn {cli_bin}: {err}")))?;
    if !status.success() {
        return Err(CliError::user(format!(
            "`{cli_bin} --install-extension` exited with {status}"
        )));
    }
    println!("installed lintropy (v{version}) into {cli_bin}");
    drop(owned_tmp);
    Ok(EXIT_OK)
}

/// Download `lintropy-<version>.vsix` into a self-deleting tempfile.
fn download_vsix(version: &str) -> Result<tempfile::NamedTempFile, CliError> {
    if which("curl").is_none() {
        return Err(CliError::user(
            "curl not found in PATH — install curl or pass --vsix PATH to skip the download",
        ));
    }

    let url = format!("{REPO_URL}/releases/download/v{version}/lintropy-{version}.vsix");
    let tmp = tempfile::Builder::new()
        .prefix("lintropy-lsp-")
        .suffix(".vsix")
        .tempfile()
        .map_err(|err| CliError::internal(format!("tempfile: {err}")))?;

    eprintln!("downloading {url}");
    let status = Command::new("curl")
        .args(["-fsSL", "--retry", "3", "-o"])
        .arg(tmp.path())
        .arg(&url)
        .status()
        .map_err(|err| CliError::internal(format!("spawn curl: {err}")))?;
    if !status.success() {
        return Err(CliError::user(format!(
            "download failed for {url} (exit {status}). \
             The release may not exist yet — pass --version to pin to a released version."
        )));
    }
    Ok(tmp)
}

fn which(bin: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path)
        .map(|dir| dir.join(bin))
        .find(|p| p.is_file())
}
