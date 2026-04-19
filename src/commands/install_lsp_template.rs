//! Back-end for `lintropy install jetbrains` — unpack the embedded
//! LSP4IJ custom template into a user-chosen directory so JetBrains
//! IDEs can import it via **LSP Console → `+` → Template → Import
//! from directory**.
//!
//! The template is the same directory that lives in
//! `editors/jetbrains/lsp4ij-template/` for contributors with a repo
//! checkout; this command materialises it for end users who only have
//! the shipped binary.

use std::fs;
use std::path::{Path, PathBuf};

use crate::commands::current_dir;
use crate::editor_assets::{LSP4IJ_TEMPLATE_DIR, LSP4IJ_TEMPLATE_DIR_NAME};
use crate::exit::{CliError, EXIT_OK};

pub(crate) fn install_jetbrains(dir: Option<PathBuf>, force: bool) -> Result<u8, CliError> {
    let parent = match dir {
        Some(p) => p,
        None => current_dir()?,
    };
    let target = parent.join(LSP4IJ_TEMPLATE_DIR_NAME);

    if target.exists() {
        if force {
            fs::remove_dir_all(&target)?;
        } else {
            return Err(CliError::user(format!(
                "refusing to overwrite existing {} (pass --force)",
                target.display()
            )));
        }
    }

    extract_dir(&target)?;

    println!("extracted {}", target.display());
    println!();
    println!("Next step — in your JetBrains IDE:");
    println!(
        "  View → Tool Windows → LSP Console → + → New Language Server → Template → Import from directory..."
    );
    println!("  Select: {}", target.display());
    Ok(EXIT_OK)
}

fn extract_dir(target: &Path) -> Result<(), CliError> {
    fs::create_dir_all(target)?;
    for file in LSP4IJ_TEMPLATE_DIR.files() {
        write_embedded_file(target, file.path(), file.contents())?;
    }
    for dir in LSP4IJ_TEMPLATE_DIR.dirs() {
        walk_dir(target, dir)?;
    }
    Ok(())
}

fn walk_dir(target: &Path, dir: &include_dir::Dir<'_>) -> Result<(), CliError> {
    for file in dir.files() {
        write_embedded_file(target, file.path(), file.contents())?;
    }
    for sub in dir.dirs() {
        walk_dir(target, sub)?;
    }
    Ok(())
}

fn write_embedded_file(target: &Path, rel: &Path, bytes: &[u8]) -> Result<(), CliError> {
    let stripped = rel.strip_prefix(LSP4IJ_TEMPLATE_DIR_NAME).unwrap_or(rel);
    let out = target.join(stripped);
    if let Some(parent) = out.parent().filter(|p| !p.as_os_str().is_empty()) {
        fs::create_dir_all(parent)?;
    }
    fs::write(&out, bytes)?;
    Ok(())
}
