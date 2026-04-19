//! Back-end for `lintropy install codex` — write a local Codex
//! marketplace root that can be added with `codex marketplace add`.
//!
//! Codex expects a marketplace root directory containing
//! `.agents/plugins/marketplace.json`, with plugin bundles referenced from
//! that manifest. For local development we generate a self-contained
//! marketplace directory so users can point Codex at it directly.

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{json, Value};

use crate::commands::current_dir;
use crate::exit::{CliError, EXIT_OK};
use crate::skill::{report_skill, write_skill};

pub const MARKETPLACE_DIR_NAME: &str = "lintropy-codex-marketplace";
pub const PLUGIN_DIR_NAME: &str = "lintropy";
const README: &str = include_str!("../../../editors/codex/README.md");

#[derive(Debug, Default)]
pub(crate) struct CodexInstall {
    pub dir: Option<PathBuf>,
    pub force: bool,
}

pub(crate) fn run(args: CodexInstall) -> Result<u8, CliError> {
    let target = resolve_target(&args)?;
    prepare_target(&target, args.force)?;

    write_marketplace(&target)?;

    println!("extracted {}", target.display());
    println!();
    println!("Next step — add the local marketplace to Codex:");
    println!("  codex marketplace add {}", target.display());
    println!();
    println!("Then install the plugin from that marketplace.");

    Ok(EXIT_OK)
}

fn write_marketplace(target: &Path) -> Result<(), CliError> {
    let marketplace_dir = target.join(".agents").join("plugins");
    fs::create_dir_all(&marketplace_dir)?;

    let plugin_root = target.join("plugins").join(PLUGIN_DIR_NAME);
    fs::create_dir_all(&plugin_root)?;
    write_plugin(&plugin_root, &build_plugin_manifest())?;
    install_bundled_skill(&plugin_root)?;

    let pretty = serde_json::to_string_pretty(&build_marketplace_manifest("./plugins/lintropy"))
        .map_err(|err| CliError::internal(format!("serialise marketplace.json: {err}")))?;
    fs::write(
        marketplace_dir.join("marketplace.json"),
        format!("{pretty}\n"),
    )?;
    fs::write(target.join("README.md"), README)?;
    Ok(())
}

fn install_bundled_skill(plugin_dir: &Path) -> Result<(), CliError> {
    let target = plugin_dir.join("skills").join("lintropy").join("SKILL.md");
    let outcome = write_skill(&target)?;
    report_skill(&target, outcome);
    Ok(())
}

fn resolve_target(args: &CodexInstall) -> Result<PathBuf, CliError> {
    let parent = match args.dir.as_ref() {
        Some(p) => p.clone(),
        None => current_dir()?,
    };
    Ok(parent.join(MARKETPLACE_DIR_NAME))
}

fn prepare_target(target: &Path, force: bool) -> Result<(), CliError> {
    if target.exists() {
        if force {
            fs::remove_dir_all(target)?;
        } else {
            return Err(CliError::user(format!(
                "refusing to overwrite existing {} (pass --force)",
                target.display()
            )));
        }
    }
    Ok(())
}

fn write_plugin(target: &Path, manifest: &Value) -> Result<(), CliError> {
    let plugin_dir = target.join(".codex-plugin");
    fs::create_dir_all(&plugin_dir)?;
    let pretty = serde_json::to_string_pretty(manifest)
        .map_err(|err| CliError::internal(format!("serialise plugin.json: {err}")))?;
    fs::write(plugin_dir.join("plugin.json"), format!("{pretty}\n"))?;
    fs::write(target.join("README.md"), README)?;
    Ok(())
}

pub fn build_plugin_manifest() -> Value {
    json!({
        "name": "lintropy",
        "version": env!("CARGO_PKG_VERSION"),
        "description": "Bundles the lintropy skill for Codex so agents can write, debug, and run lintropy rules with repo-local guidance.",
        "author": {
            "name": "rns70 & Typiqally",
            "email": "12190745+Typiqally@users.noreply.github.com"
        },
        "homepage": "https://github.com/Typiqally/lintropy",
        "repository": "https://github.com/Typiqally/lintropy",
        "license": "MIT",
        "keywords": [
            "lintropy",
            "linting",
            "tree-sitter",
            "skills",
            "codex"
        ],
        "skills": "./skills/",
        "interface": {
            "displayName": "Lintropy",
            "shortDescription": "Lintropy rule authoring and debugging workflows for Codex",
            "longDescription": "Use lintropy from Codex to write or debug tree-sitter-based lint rules, inspect ASTs with `lintropy ts-parse`, validate configs, and run targeted `lintropy check` loops inside the repo.",
            "developerName": "Typiqally",
            "category": "Coding",
            "capabilities": ["Interactive", "Read", "Write"],
            "websiteURL": "https://github.com/Typiqally/lintropy",
            "defaultPrompt": "Use lintropy to inspect this repo's rules, write or debug a lintropy rule, and run lintropy checks to verify the change",
            "brandColor": "#1d4ed8"
        }
    })
}

pub fn build_marketplace_manifest(path: &str) -> Value {
    json!({
        "name": "lintropy",
        "owner": {
            "name": "rns70 & Typiqally",
            "email": "12190745+Typiqally@users.noreply.github.com"
        },
        "metadata": {
            "description": "Lintropy's Codex integrations.",
            "version": env!("CARGO_PKG_VERSION")
        },
        "plugins": [
            {
                "name": "lintropy",
                "source": {
                    "source": "local",
                    "path": path
                },
                "description": "Bundles the lintropy skill for Codex so agents can write, debug, and run lintropy rules with repo-local guidance.",
                "version": env!("CARGO_PKG_VERSION"),
                "homepage": "https://github.com/Typiqally/lintropy",
                "repository": "https://github.com/Typiqally/lintropy",
                "license": "MIT",
                "keywords": [
                    "skills",
                    "linter",
                    "tree-sitter"
                ]
            }
        ]
    })
}
