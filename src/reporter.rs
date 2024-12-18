#[cfg(feature = "lsp")]
mod lsp;
#[cfg(feature = "lsp")]
pub use lsp::*;

#[cfg(feature = "serial")]
mod serial;
#[cfg(feature = "serial")]
pub use serial::*;
use slotmap::new_key_type;

use crate::{diagnostic::Diagnostic, span::Span};

new_key_type! { pub struct LookupKey; }

pub trait Reporter {
    fn report(&mut self, diagnostic: Diagnostic);
    fn report_all(&mut self, diagnostics: Vec<Diagnostic>) {
        for diagnostic in diagnostics {
            self.report(diagnostic);
        }
    }

    fn has_errors(&self) -> bool;
    fn is_empty(&self) -> bool;

    fn eof_span(&self, key: LookupKey) -> Span;
}
