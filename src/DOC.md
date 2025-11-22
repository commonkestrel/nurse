Nurse is a diagnostic formatting library designed for internal use in compilers, interpreters, and assemblers.
Designed after the Rust compiler's error message formatting,
this crate allows user-facing text-processors to provide users with detailed and informative error messages.

## Example

![Example terminal formatting, with multiline spans!](https://raw.githubusercontent.com/commonkestrel/nurse/refs/heads/main/misc/terminal_example.png)
```rust
use nurse::prelude::*;
use std::io;

fn main() -> io::Result<()> {
    let file = "fn main() {
    println!(\"hello world\";\n\
    }";

    // Create a new reporter and register the example file
    let mut reporter = TerminalReporter::default();
    let lookup = reporter.register_file("example.txt", file);
    
    // Create spans for `(` and `"hello world"` based on character positions
    let open_paren_span = Span::new(lookup, 24..25);
    let fn_span = Span::new(lookup, 10..41);

    // Report messages to be output
    reporter.report(error!(open_paren_span, "missing closing parenthesis"));
    reporter.report(debug!(fn_span, "code block found"));
    reporter.report(error!("unable to compile due to previous errors"));

    // Emit all messages that were previously reported to stdout
    reporter.emit_all(&mut io::stdout())
}
```

## Features

There are a number of features that allow for additional functionality,
and are as follows:

- `terminal`: Enabled by default. Allows for diagnostics to be formatted and printed to a writer, commonly stdout.
- `lsp`: Allows for diagnostics to be reported as a language server message.
- `smol`: Allows for many of the I/O writing operations to be async using [`smol`](https://github.com/smol-rs/smol).
- `tokio`: Allows for many of the I/O writing operations to be async using [`tokio`](https://github.com/tokio-rs/tokio)
