#[cfg(feature = "lsp")]
pub mod lsp;
#[cfg(feature = "lsp")]
pub use lsp::*;

#[cfg(feature = "terminal")]
pub mod terminal;
#[cfg(feature = "terminal")]
pub use terminal::*;

use slotmap::new_key_type;
new_key_type! {
    /// A key used to identify a file in a [`Reporter`]'s lookup table.
    /// Will lead to panics if used with a `Reporter` other than the origin.
    pub struct LookupKey;
}
