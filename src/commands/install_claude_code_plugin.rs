//! Back-end for `lintropy lsp install claude-code` — write a Claude Code
//! plugin directory that registers `lintropy lsp` as a Language Server.
//!
//! The plugin manifest is generated at extract time, not embedded, so:
//!
//! - `version` tracks `CARGO_PKG_VERSION`.
//! - The `extensionToLanguage` map is gated by the same `cfg(feature = ...)`
//!   flags that gate `crate::langs::Language::from_name`. If the binary
//!   was built without `lang-go`, `.go` is not mapped, and Claude Code
//!   won't start the LSP for Go buffers (which would produce no
//!   diagnostics anyway).
//! - `command` is resolved to an absolute path via `which` so Claude
//!   Code's subprocess env does not have to match the invoking shell's
//!   `PATH`. Users can still hand-edit the emitted `plugin.json` if they
//!   want to pin a different binary.
//!
//! After writing the plugin directory we optionally shell out to
//! `claude plugin install <dir> --scope <scope>` so the user does not
//! have to run a second command. Pass `--no-install` to skip the shell-out
//! (useful in CI and for users who prefer to use the marketplace flow).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command as ProcessCommand;

use serde_json::{json, Value};

use crate::cli::PluginScope;
use crate::commands::current_dir;
use crate::exit::{CliError, EXIT_OK};
use crate::skill::{report_skill, write_skill};

pub const PLUGIN_DIR_NAME: &str = "lintropy-claude-code-plugin";
pub const PLUGIN_NAME: &str = "lintropy-lsp";
const README: &str = include_str!("../../editors/claude-code/README.md");

#[derive(Debug, Default)]
pub(crate) struct ClaudeCodeInstall {
    pub dir: Option<PathBuf>,
    pub force: bool,
    pub scope: PluginScope,
    pub no_install: bool,
    pub with_skill: bool,
}

pub(crate) fn run(args: ClaudeCodeInstall) -> Result<u8, CliError> {
    let target = resolve_target(&args)?;
    prepare_target(&target, args.force)?;

    let command = resolve_lintropy_binary();
    let manifest = build_manifest(&command);
    write_plugin(&target, &manifest)?;

    println!("extracted {}", target.display());

    if args.with_skill {
        install_skill_for_scope(args.scope)?;
    }

    if args.no_install {
        println!();
        println!("Next step — install the plugin:");
        println!(
            "  claude plugin install {} --scope {}",
            target.display(),
            args.scope.as_str()
        );
        return Ok(EXIT_OK);
    }

    if let Some(claude) = locate_claude_cli() {
        run_claude_install(&claude, &target, args.scope)?;
    } else {
        println!();
        println!("`claude` CLI not found on PATH. Run the install manually:");
        println!(
            "  claude plugin install {} --scope {}",
            target.display(),
            args.scope.as_str()
        );
    }
    Ok(EXIT_OK)
}

fn install_skill_for_scope(scope: PluginScope) -> Result<(), CliError> {
    let root = match scope {
        PluginScope::Project => current_dir()?,
        PluginScope::User => home_dir()?,
    };
    let target = root
        .join(".claude")
        .join("skills")
        .join("lintropy")
        .join("SKILL.md");
    let outcome = write_skill(&target)?;
    report_skill(&target, outcome);
    Ok(())
}

fn home_dir() -> Result<PathBuf, CliError> {
    if let Some(home) = std::env::var_os("HOME") {
        if !home.is_empty() {
            return Ok(PathBuf::from(home));
        }
    }
    #[cfg(windows)]
    {
        if let Some(profile) = std::env::var_os("USERPROFILE") {
            if !profile.is_empty() {
                return Ok(PathBuf::from(profile));
            }
        }
    }
    Err(CliError::user(
        "could not resolve home directory — pass `--scope project` or set HOME explicitly",
    ))
}

fn resolve_target(args: &ClaudeCodeInstall) -> Result<PathBuf, CliError> {
    let parent = match args.dir.as_ref() {
        Some(p) => p.clone(),
        None => current_dir()?,
    };
    Ok(parent.join(PLUGIN_DIR_NAME))
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
    let plugin_dir = target.join(".claude-plugin");
    fs::create_dir_all(&plugin_dir)?;
    let pretty = serde_json::to_string_pretty(manifest)
        .map_err(|err| CliError::internal(format!("serialise plugin.json: {err}")))?;
    fs::write(plugin_dir.join("plugin.json"), format!("{pretty}\n"))?;
    fs::write(target.join("README.md"), README)?;
    Ok(())
}

/// Build the plugin manifest with a specific `command` string.
///
/// The `extensionToLanguage` map is feature-gated so the emitted plugin
/// only activates the LSP for languages the binary was compiled with.
pub fn build_manifest(command: &str) -> Value {
    let mut ext = serde_json::Map::<String, Value>::new();
    ext.insert(".yaml".into(), json!("yaml"));
    ext.insert(".yml".into(), json!("yaml"));
    ext.insert(".rs".into(), json!("rust"));
    #[cfg(feature = "lang-go")]
    {
        ext.insert(".go".into(), json!("go"));
    }
    #[cfg(feature = "lang-python")]
    {
        ext.insert(".py".into(), json!("python"));
        ext.insert(".pyi".into(), json!("python"));
    }
    #[cfg(feature = "lang-typescript")]
    {
        ext.insert(".ts".into(), json!("typescript"));
        ext.insert(".tsx".into(), json!("typescriptreact"));
        ext.insert(".mts".into(), json!("typescript"));
        ext.insert(".cts".into(), json!("typescript"));
    }

    json!({
        "name": PLUGIN_NAME,
        "version": env!("CARGO_PKG_VERSION"),
        "description": "Registers `lintropy lsp` as a Language Server so Claude Code sees lintropy diagnostics live while editing.",
        "homepage": "https://github.com/Typiqally/lintropy",
        "repository": "https://github.com/Typiqally/lintropy",
        "license": "MIT",
        "lspServers": {
            "lintropy": {
                "command": command,
                "args": ["lsp"],
                "extensionToLanguage": ext
            }
        }
    })
}

/// Resolve the `lintropy` binary to an absolute path if possible. Falls
/// back to the literal name `"lintropy"` so the emitted manifest is still
/// usable when `which` fails.
fn resolve_lintropy_binary() -> String {
    if let Ok(current) = std::env::current_exe() {
        if let Ok(canonical) = current.canonicalize() {
            if let Some(name) = canonical.file_name().and_then(|s| s.to_str()) {
                if name == "lintropy" || name == "lintropy.exe" {
                    return canonical.to_string_lossy().into_owned();
                }
            }
        }
    }
    if let Some(found) = which_on_path("lintropy") {
        return found.to_string_lossy().into_owned();
    }
    "lintropy".to_string()
}

fn which_on_path(binary: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path) {
        let candidate = dir.join(binary);
        if candidate.is_file() {
            return Some(candidate);
        }
        #[cfg(windows)]
        {
            let exe = dir.join(format!("{binary}.exe"));
            if exe.is_file() {
                return Some(exe);
            }
        }
    }
    None
}

fn locate_claude_cli() -> Option<PathBuf> {
    which_on_path("claude")
}

fn run_claude_install(
    claude: &Path,
    plugin_dir: &Path,
    scope: PluginScope,
) -> Result<(), CliError> {
    println!();
    println!(
        "running: {} plugin install {} --scope {}",
        claude.display(),
        plugin_dir.display(),
        scope.as_str()
    );
    let status = ProcessCommand::new(claude)
        .arg("plugin")
        .arg("install")
        .arg(plugin_dir)
        .arg("--scope")
        .arg(scope.as_str())
        .status()
        .map_err(|err| {
            CliError::internal(format!("failed to launch `{}`: {err}", claude.display()))
        })?;
    if !status.success() {
        return Err(CliError::user(format!(
            "`claude plugin install` exited with status {status}"
        )));
    }
    Ok(())
}
