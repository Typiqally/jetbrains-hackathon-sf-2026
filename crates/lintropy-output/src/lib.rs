//! Reporter trait. Concrete text + JSON reporters land in WP3.

use lintropy_core::{Diagnostic, Result, Summary};

/// Sink that turns diagnostics + summary into human or machine output.
///
/// Every reporter writes through a `Box<dyn Write>` so `--output` can
/// swap the destination without changing reporter types (§7.7 of the spec).
pub trait Reporter {
    /// Emit the diagnostics and summary. Called once per `lintropy check` run.
    fn report(&mut self, diagnostics: &[Diagnostic], summary: &Summary) -> Result<()>;
}
