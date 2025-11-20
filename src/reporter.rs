#[cfg(feature = "lsp")]
mod lsp;
#[cfg(feature = "lsp")]
pub type Reporter = lsp::LspReporter;

#[cfg(feature = "serial")]
mod serial;
#[cfg(feature = "serial")]
pub type Reporter = serial::SerialReporter;

use slotmap::new_key_type;
new_key_type! { pub struct LookupKey; }
