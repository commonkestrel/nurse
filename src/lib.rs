#![doc = include_str!("DOC.md")]
#![warn(missing_docs)]

mod diagnostic;
mod lookup;
mod reporter;
mod span;

pub use diagnostic::*;
pub use reporter::*;
pub use span::*;

/// Re-exports most of the commonly used st ructures and macros from the crate.
/// Designed to be used as a glob import (e.g. `use nurse::prelude::*`)
pub mod prelude {
    pub use crate::diagnostic::_debug as debug;
    pub use crate::diagnostic::_error as error;
    pub use crate::diagnostic::_info as info;
    pub use crate::diagnostic::_warning as warning;
    pub use crate::diagnostic::{Diagnostic, LevelFilter};
    #[cfg(feature = "lsp")]
    pub use crate::reporter::LspReporter;
    #[cfg(feature = "terminal")]
    pub use crate::reporter::TerminalReporter;
    pub use crate::span::{Span, Spanned};
}

#[cfg(all(feature = "smol", feature = "tokio"))]
compile_error!("you may enable either the `smol` or `tokio` features, but not both");
