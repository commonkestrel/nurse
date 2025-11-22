#[cfg(feature = "lsp")]
mod lsp;
#[cfg(feature = "lsp")]
pub use lsp::*;

#[cfg(feature = "terminal")]
mod terminal;
#[cfg(feature = "terminal")]
pub use terminal::*;

use slotmap::new_key_type;
#[cfg(feature = "terminal")]
new_key_type! {
    /// A key used to identify a file in a [`TerminalReporter`]'s lookup table.
    /// 
    /// Will lead to panics if used with a reporter other than the origin.
    pub struct LookupKey;
}
