mod span;
mod diagnostic;
mod reporter;
mod lookup;

pub mod prelude {
    pub use super::span::{Span, Spanned};
    pub use super::{
        error, spanned_error,
        warning, spanned_warning,
        info, spanned_info,
        hint, spanned_hint
    };
}

#[cfg(all(feature = "async-std", feature = "tokio"))]
compile_error!("you may enable either `async-std` or `tokio`, but not both");
