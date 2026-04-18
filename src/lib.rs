//! Stub crate root for the workspace-level integration-test package.
//!
//! The workspace `Cargo.toml` is both a virtual manifest (declaring the
//! real crates under `crates/`) and a thin `lintropy-e2e-tests` package
//! whose sole purpose is to own the cross-cutting end-to-end tests under
//! `tests/`. It ships no library and no binary.
