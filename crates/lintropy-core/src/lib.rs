//! Shared types, errors, and traits that other lintropy crates program against.
//!
//! This crate intentionally ships zero business logic. Each downstream crate
//! (`lintropy-langs`, `lintropy-rules`, `lintropy-output`, `lintropy-cli`)
//! pulls its types from here so upgrades stay coordinated.

pub mod config;
pub mod engine;
pub mod error;
pub mod predicates;
pub mod template;
pub mod types;

pub use error::{LintropyError, Result};
pub use types::{Diagnostic, FixHunk, RuleId, Severity, Span, Summary};
