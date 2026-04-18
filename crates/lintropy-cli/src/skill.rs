//! Embedded canonical `SKILL.md` + version tag.
//!
//! The source file lives at `crates/lintropy-cli/skill/SKILL.md` so the
//! `lintropy-cli` crate is self-contained when published to crates.io
//! (authored under WP6, relocated under WP9 to satisfy `cargo publish`'s
//! "files must be inside the package root" constraint). Consumed by
//! `init --with-skill` to materialise the skill into agent skill
//! directories (`.claude/skills/lintropy/`, `.cursor/skills/lintropy/`).
//!
//! `SKILL_VERSION` must match the `# version: <semver>` header on the
//! first line of `SKILL.md` — `init --with-skill` uses it to decide
//! whether to upgrade an existing file in place.

pub const EMBEDDED_SKILL: &str = include_str!("../skill/SKILL.md");
pub const SKILL_VERSION: &str = "0.2.0";
