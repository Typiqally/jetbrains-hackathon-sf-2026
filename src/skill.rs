//! Embedded canonical `SKILL.md` + version tag + the one-call helper
//! used by both `init --with-skill` and `lsp install claude-code
//! --with-skill` to materialise the skill into agent skill directories.
//!
//! `SKILL_VERSION` must match the `# version: <semver>` header on the
//! first line of `SKILL.md`; `write_skill` uses it to decide whether to
//! upgrade an existing file in place.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::exit::CliError;

pub const EMBEDDED_SKILL: &str = include_str!("../skill/SKILL.md");
pub const SKILL_VERSION: &str = "0.3.0";

#[derive(Debug, PartialEq, Eq)]
pub enum SkillOutcome {
    Created,
    Upgraded,
    Unchanged,
}

/// Write `SKILL.md` at `path`, skipping when the existing file's version
/// header already matches the embedded `SKILL_VERSION`.
pub fn write_skill(path: &Path) -> Result<SkillOutcome, CliError> {
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        fs::create_dir_all(parent)?;
    }
    let existed = path.exists();
    if existed {
        let current = fs::read_to_string(path)?;
        if version_header(&current) == Some(SKILL_VERSION) {
            return Ok(SkillOutcome::Unchanged);
        }
    }
    atomic_write(path, EMBEDDED_SKILL.as_bytes())?;
    Ok(if existed {
        SkillOutcome::Upgraded
    } else {
        SkillOutcome::Created
    })
}

pub fn report_skill(path: &Path, outcome: SkillOutcome) {
    let label = match outcome {
        SkillOutcome::Created => "created",
        SkillOutcome::Upgraded => "upgraded",
        SkillOutcome::Unchanged => "unchanged",
    };
    println!("{label} {}", path.display());
}

fn version_header(source: &str) -> Option<&str> {
    let first = source.lines().next()?;
    let rest = first.trim_start().strip_prefix('#')?.trim_start();
    rest.strip_prefix("version:").map(str::trim)
}

fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), CliError> {
    let parent: PathBuf = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    let mut tmp = tempfile::NamedTempFile::new_in(&parent)
        .map_err(|err| CliError::internal(format!("tempfile: {err}")))?;
    tmp.write_all(bytes)?;
    tmp.as_file_mut().sync_all()?;
    tmp.persist(path)
        .map_err(|err| CliError::internal(format!("persist: {err}")))?;
    Ok(())
}
