#![doc = include_str!("DOC.md")]
#![warn(missing_docs)]

pub mod diagnostic;
mod lookup;
pub mod reporter;
pub mod span;

pub mod prelude {
    pub use crate::reporter::Reporter;
    pub use crate::span::{Span, Spanned};
    pub use crate::diagnostic::Diagnostic;
    pub use crate::diagnostic::_error as error;
    pub use crate::diagnostic::_warning as warning;
    pub use crate::diagnostic::_info as info;
    pub use crate::diagnostic::_hint as hint;
    pub use crate::diagnostic::_debug as debug;
}

#[cfg(all(feature = "smol", feature = "tokio"))]
compile_error!("you may enable either `smol` or `tokio`, but not both");
