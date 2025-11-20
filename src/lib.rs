pub mod diagnostic;
mod lookup;
pub mod reporter;
pub mod span;

pub mod prelude {
    pub use crate::reporter::Reporter;
    pub use crate::span::{Span, Spanned};
}

#[cfg(all(feature = "smol", feature = "tokio"))]
compile_error!("you may enable either `smol` or `tokio`, but not both");
