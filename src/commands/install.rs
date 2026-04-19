//! `lintropy install <target>` — one-command install for every editor
//! or agent lintropy knows how to integrate with.
//!
//! - **vscode / cursor**: builds the local extension into a `.vsix` and
//!   hands it to the editor's `--install-extension` flag. With
//!   `--package-only -o <path>` the `.vsix` is written without running
//!   the editor CLI.
//! - **jetbrains**: unpacks the LSP4IJ custom template into `--dir`
//!   (defaults to cwd). Still needs one IDE-side import step.
//! - **claude-code**: generates the plugin manifest fresh (version,
//!   feature-gated extension map, absolute `command` path), materialises
//!   the lintropy skill at `.claude/skills/lintropy/SKILL.md` for the
//!   matching scope, and shells out to `claude plugin install <dir>
//!   --scope <scope>` when the `claude` CLI is on `PATH`. Pass
//!   `--no-install` to skip the shell-out and print the command instead.

use crate::cli::{InstallArgs, InstallTarget};
use crate::commands::{install_claude_code_plugin, install_lsp_extension, install_lsp_template};
use crate::exit::CliError;

pub fn run(args: InstallArgs) -> Result<u8, CliError> {
    match args.target {
        InstallTarget::Vscode => install_vsix(args, install_lsp_extension::VsixEditor::Vscode),
        InstallTarget::Cursor => install_vsix(args, install_lsp_extension::VsixEditor::Cursor),
        InstallTarget::Jetbrains => install_lsp_template::install_jetbrains(args.dir, args.force),
        InstallTarget::ClaudeCode => {
            install_claude_code_plugin::run(install_claude_code_plugin::ClaudeCodeInstall {
                dir: args.dir,
                force: args.force,
                scope: args.scope,
                no_install: args.no_install,
            })
        }
    }
}

fn install_vsix(
    args: InstallArgs,
    editor: install_lsp_extension::VsixEditor,
) -> Result<u8, CliError> {
    install_lsp_extension::run(install_lsp_extension::VsixBuild {
        editor: Some(editor),
        profile: args.profile,
        package_only: args.package_only,
        output: args.output,
    })
}
