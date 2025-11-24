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
    reporter.report(warning!("something warning"));
    reporter.report(info!("something info"));

    // Emit all messages that were previously reported to stdout
    reporter.emit_all()
}
