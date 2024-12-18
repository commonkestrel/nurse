pub mod diagnostic;
mod lookup;
pub mod reporter;
pub mod span;

pub mod prelude {
    pub use super::span::{Span, Spanned};
    pub use super::{
        error, hint, info, spanned_error, spanned_hint, spanned_info, spanned_warning, warning,
    };
}

#[cfg(all(feature = "async-std", feature = "tokio"))]
compile_error!("you may enable either `async-std` or `tokio`, but not both");
