use anstream::{
    stream::{AsLockedWrite, RawStream},
    AutoStream,
};
use colored::{Color, Colorize};
use slotmap::SlotMap;
#[cfg(not(feature = "smol"))]
use std::io::{self, Write};

#[cfg(feature = "smol")]
use smol::{io::AsyncWriteExt, lock::Mutex, Unblock};

use crate::{
    diagnostic::{Diagnostic, LevelFilter},
    lookup::{Location, Lookup},
    span::Span,
    Note,
};

use super::LookupKey;

#[cfg(not(feature = "smol"))]
type Emitter<T> = AutoStream<T>;

#[cfg(feature = "smol")]
type Emitter<T> = Unblock<AutoStream<T>>;

fn new_emitter<T: RawStream>(emitter: T) -> Emitter<T> {
    #[cfg(not(feature = "smol"))]
    return AutoStream::auto(emitter);
    #[cfg(feature = "smol")]
    return Unblock::new(AutoStream::auto(emitter));
}

/// A reporter that formats and displays reported diagnostics
/// to the terminal.
#[cfg(not(feature = "smol"))]
#[derive(Debug)]
pub struct TerminalReporter<T: RawStream + AsLockedWrite> {
    diagnostics: Vec<Diagnostic>,
    lookups: SlotMap<LookupKey, (String, Lookup)>,
    filter: LevelFilter,
    emitter: Emitter<T>,
}

#[cfg(feature = "smol")]
/// A reporter that formats and displays reported diagnostics
/// to the terminal.
#[derive(Debug)]
pub struct TerminalReporter<T: RawStream + AsLockedWrite + Send + 'static> {
    diagnostics: Mutex<Vec<Diagnostic>>,
    lookups: Mutex<SlotMap<LookupKey, (String, Lookup)>>,
    filter: LevelFilter,
    emitter: Emitter<T>,
}

impl<T: RawStream + AsLockedWrite + Send> TerminalReporter<T> {
    /// Creates an empty `TerminalReporter` with the given emitter.
    pub fn new(emitter: T) -> TerminalReporter<T> {
        TerminalReporter {
            diagnostics: Vec::new().into(),
            lookups: SlotMap::with_key().into(),
            filter: LevelFilter::Debug,
            emitter: new_emitter(emitter),
        }
    }

    /// Creates an empty `TerminalReporter` with the given emitter and filter level.
    pub fn filtered(emitter: T, filter: LevelFilter) -> TerminalReporter<T> {
        TerminalReporter {
            diagnostics: Vec::new().into(),
            lookups: SlotMap::with_key().into(),
            filter,
            emitter: new_emitter(emitter),
        }
    }
}

#[cfg(not(feature = "smol"))]
impl<T: RawStream + AsLockedWrite> TerminalReporter<T> {
    /// Inserts a file into the lookup table with the given filename and contents,
    /// returning the [`LookupKey`] associated with it.
    /// This lookup key must only be used with the reporter it was registered with.
    ///
    /// This operation can be computationally intensive,
    /// depending on the file size.
    pub fn register_file<N: ToString, F: ToString>(&mut self, name: N, contents: F) -> LookupKey {
        self.lookups
            .insert((name.to_string(), Lookup::new(contents.to_string())))
    }

    /// Prints a diagnostic to the given emitter,
    /// generally [`Stdout`](std::io::Stdout).
    pub fn emit(&mut self, diagnostic: Diagnostic) -> io::Result<()> {
        if !self.filter.passes(diagnostic.level) {
            return Ok(());
        }

        self.emit_fancy(diagnostic)
    }

    /// Prints all reported diagnostics to the provided `emitter`,
    /// generally [`Stdout`](std::io::Stdout).
    ///
    /// Clears the store of reported diagnostics,
    /// causing subsequent calls not to repeat already emitted diagnostics.
    pub fn emit_all(&mut self) -> io::Result<()> {
        let mut result = Ok(());

        let mut diagnostics = Vec::new();
        std::mem::swap(&mut diagnostics, &mut self.diagnostics);

        for diagnostic in diagnostics {
            if !self.filter.passes(diagnostic.level) {
                continue;
            }

            if let Err(err) = self.emit_fancy(diagnostic) {
                result = Err(err);
            }
        }

        result
    }

    fn emit_fancy(&mut self, diagnostic: Diagnostic) -> io::Result<()> {
        let mut note_offset = diagnostic.level.title().len() + 1;
        let message = diagnostic.format_message();
        writeln!(self.emitter, "{message}")?;

        let note = match diagnostic.note {
            Some(Note {
                ref value,
                span: Some(span),
            }) => Some((value.as_str(), span)),
            _ => None,
        };

        if let Some(span) = diagnostic.span {
            let (pointer, offset) =
                Self::pointer(&self.lookups, span, note, diagnostic.level.color());
            note_offset = offset + 1;
            writeln!(self.emitter, "{pointer}")?;
        }

        let note = diagnostic.note.is_some();
        if let Some(Note { value, span: None }) = diagnostic.note {
            writeln!(
                self.emitter,
                "{:>note_offset$} {}: {}",
                "=".bright_blue().bold(),
                "note".bold(),
                value
            )?;
        }

        if diagnostic.span.is_some() || note {
            writeln!(self.emitter)?;
        }

        Ok(())
    }

    fn pointer(
        lookups: &SlotMap<LookupKey, (String, Lookup)>,
        span: Span,
        note: Option<(&str, Span)>,
        arrow_color: Color,
    ) -> (String, usize) {
        let (file, lookup) = lookups.get(span.lookup()).expect("span should ");

        let lines = lookup.lines(span.start()..span.end());
        let line_n = lines.start + 1;
        let col_n = lookup.col_from_line(lines.start, span.start()) + 1;

        if lines.len() > 1 {
            let start = lookup.line(lines.start).trim_end();
            let end = lookup.line(lines.end - 1).trim_end();

            let end_col = lookup.col_from_line(lines.end - 1, span.end());

            let offset = ((lines.end + 1).ilog10()).max(2) as usize + 2;

            (
                format!(
                    "\
                    {arrow:>arr_space$} [{name}:{line_n}:{col_n}]\n\
                    {cap:>width$}\n\
                    {start_n}   {start}\n\
                    {cap:>width$} {start_pointer}\n\
                    {dot_n}  {pipe}\n\
                    {end_n} {pipe} {end}\n\
                    {cap:>width$} {end_pointer}\n\
                    ",
                    arrow = "——>".bright_blue().bold(),
                    arr_space = offset + 2,
                    name = file.bold().bright_cyan().underline(),
                    cap = "┃".bright_blue().bold(),
                    width = offset + 1,
                    start_n = format!("{:<offset$}┃", lines.start + 1)
                        .bright_blue()
                        .bold(),
                    dot_n = format!("{:<offset$}", "...").bright_blue().bold(),
                    end_n = format!("{:<offset$}┃", lines.end + 1).bright_blue().bold(),
                    start_pointer = format!(
                        "╭─{blank:·>start$}{blank:—>length$}",
                        blank = "",
                        start = col_n - 1,
                        length = start.len() - col_n + 1,
                    )
                    .color(arrow_color),
                    end_pointer = format!("╰─{blank:─>length$}", blank = "", length = end_col)
                        .color(arrow_color),
                    pipe = "│".color(arrow_color)
                ),
                offset,
            )
        } else {
            let line = lookup.line(lines.start).trim_end();
            let offset = (lines.start + 1).ilog10() as usize + 2;

            (
                format!(
                    "\
                    {arrow:>arr_space$} [{name}:{line_n}:{col_n}]\n\
                    {cap:>width$}\n\
                    {n}{cap} {line}\n\
                    {cap:>width$} {pointer}\
                    ",
                    arrow = "——>".bright_blue().bold(),
                    name = file.bold().bright_cyan().underline(),
                    cap = "┃".bright_blue().bold(),
                    width = offset + 1,
                    arr_space = offset + 2,
                    n = format!("{line_n:<offset$}").bright_blue().bold(),
                    pointer = format!(
                        "{blank:>start$}{blank:‾>length$}",
                        blank = "",
                        start = col_n - 1,
                        length = span.end() - span.start(),
                    )
                    .bold()
                    .color(arrow_color),
                ),
                offset,
            )
        }
    }

    /// Gets the line-column location of the span in its file.
    ///
    /// ## Panics
    ///
    /// This function will panic if `span` refers to a span in a file not registered with this reporter, e.g.
    ///
    /// ```should_panic
    /// # use nurse::prelude::*;
    /// let mut reporter1 = TerminalReporter::default();
    /// let reporter2 = TerminalReporter::default();
    ///
    /// let key = reporter1.register_file("example.txt", r#""hello world""#);
    /// let span = Span::new(key, 0..1);
    /// // Should panic!
    /// reporter2.location(span);
    /// ```
    pub fn location(&self, span: Span) -> Location {
        let lookup = &self
            .lookups
            .get(span.lookup())
            .expect("span should refer to an already registered file")
            .1;

        let (line, column) = lookup.line_col(span.start());
        Location { line, column }
    }

    /// Adds the provided `diagnostic` to the inner collection.
    ///
    /// Will be emitted when [`emit_all`](TerminalReporter::emit_all) is called.
    pub fn report(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    /// Adds the provided list of `diagnostics` to the inner collection.
    ///
    /// All will be emitted when [`emit_all`](TerminalReporter::emit_all) is called.
    pub fn report_all(&mut self, mut diagnostics: Vec<Diagnostic>) {
        self.diagnostics.append(&mut diagnostics);
    }

    /// Returns `true` if any diagnostics in the inner collection are of level [`Error`](crate::Level::Error),
    /// otherwise returns `false`.
    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|diagnostic| diagnostic.is_error())
    }

    /// Returns `true` if there are no diagnostics stored in the inner collection,
    /// otherwise returns `false`.
    pub fn is_empty(&self) -> bool {
        self.diagnostics.is_empty()
    }

    /// Returns a single character-wide span at the end of the file referred to by `key`.
    ///
    /// This is useful raising errors if you expect a token,
    /// but instead find the end of a file.
    ///
    /// ## Panics
    ///
    /// This function will panic if `key` refers to a file not registered with this reporter, e.g.
    ///
    /// ```should_panic
    /// # use nurse::TerminalReporter;
    /// let mut reporter1 = TerminalReporter::default();
    /// let reporter2 = TerminalReporter::default();
    ///
    /// let key = reporter1.register_file("example.txt", r#""hello world""#);
    /// // Should panic!
    /// reporter2.eof_span(key);
    /// ```
    ///
    /// ## Example
    ///
    /// ```rust
    /// # use nurse::prelude::*;
    /// # let mut reporter = TerminalReporter::default();
    /// let key = reporter.register_file("example.txt", "3 + ");
    /// let eof_span = reporter.eof_span(key);
    ///
    /// reporter.report(error!(eof_span, "expected token, found EOF"));
    /// ```
    pub fn eof_span(&self, key: LookupKey) -> Span {
        let (_, lookup) = self
            .lookups
            .get(key)
            .expect("key should refer to an already registered file");

        let eof = lookup.file_len();

        Span {
            start: eof,
            end: eof + 1,
            lookup: key,
        }
    }
}

#[cfg(feature = "smol")]
impl<T: RawStream + AsLockedWrite + Send + 'static> TerminalReporter<T> {
    /// Inserts a file into the lookup table with the given filename and contents,
    /// returning the [`LookupKey`] associated with it.
    /// This lookup key must only be used with the reporter it was registered with.
    ///
    /// This operation can be computationally intensive,
    /// depending on the file size.
    pub async fn register_file<N: ToString, F: ToString>(&self, name: N, contents: F) -> LookupKey {
        let mut lookups = self.lookups.lock().await;
        lookups.insert((name.to_string(), Lookup::new(contents.to_string())))
    }

    // Adds the provided `diagnostic` to the inner collection.
    ///
    /// Will be emitted when [`emit_all`](TerminalReporter::emit_all) is called.
    pub async fn report(&self, diagnostic: Diagnostic) {
        let mut diagnostics = self.diagnostics.lock().await;
        diagnostics.push(diagnostic);
    }

    /// Adds the provided list of `diagnostics` to the inner collection.
    ///
    /// All will be emitted when [`emit_all`](TerminalReporter::emit_all) is called.
    pub async fn report_all(&self, mut diagnostics: Vec<Diagnostic>) {
        let mut reported = self.diagnostics.lock().await;
        reported.append(&mut diagnostics);
    }

    /// Returns `true` if any diagnostics in the inner collection are of level [`Error`](crate::Level::Error),
    /// otherwise returns `false`.
    pub async fn has_errors(&self) -> bool {
        let diagnostics = self.diagnostics.lock().await;
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.is_error())
    }

    /// Returns `true` if there are no diagnostics stored in the inner collection,
    /// otherwise returns `false`.
    pub async fn is_empty(&self) -> bool {
        let diagnostics = self.diagnostics.lock().await;
        diagnostics.is_empty()
    }

    /// Prints a diagnostic to the internal emitter, `stdout` by default.
    pub async fn emit(&mut self, diagnostic: Diagnostic) -> std::io::Result<()> {
        if !self.filter.passes(diagnostic.level) {
            return Ok(());
        }

        self.emit_fancy(diagnostic).await
    }

    /// Prints all reported diagnostics to the internal emitter, `stdout` by default.
    ///
    /// Clears the store of reported diagnostics,
    /// causing subsequent calls not to repeat already emitted diagnostics.
    pub async fn emit_all(&mut self) -> std::io::Result<()> {
        let mut result = Ok(());

        let mut diagnostics = Vec::new();
        let mut owned_diagnostics = self.diagnostics.lock().await;

        std::mem::swap(&mut diagnostics, &mut owned_diagnostics);
        std::mem::drop(owned_diagnostics);

        for diagnostic in diagnostics.into_iter() {
            if !self.filter.passes(diagnostic.level) {
                continue;
            }

            if let Err(err) = self.emit_fancy(diagnostic).await {
                result = Err(err);
            }
        }

        result
    }

    async fn emit_fancy(&mut self, diagnostic: Diagnostic) -> std::io::Result<()> {
        let mut note_offset = diagnostic.level.title().len() + 1;
        let message = diagnostic.format_message();
        self.emitter.write(format!("{message}").as_bytes()).await?;

        let note = match diagnostic.note {
            Some(Note {
                ref value,
                span: Some(span),
            }) => Some((value.as_str(), span)),
            _ => None,
        };

        if let Some(span) = diagnostic.span {
            let (pointer, offset) =
                self.pointer(span, note, diagnostic.level.color()).await;
            note_offset = offset + 1;
            self.emitter
                .write_all(format!("{pointer}").as_bytes())
                .await?;
        }

        let note = diagnostic.note.is_some();
        if let Some(Note { value, span: None }) = diagnostic.note {
            self.emitter
                .write_all(
                    format!(
                        "{:>note_offset$} {}: {}",
                        "=".bright_blue().bold(),
                        "note".bold(),
                        value
                    )
                    .as_bytes(),
                )
                .await?;
        }

        if diagnostic.span.is_some() || note {
            self.emitter.write(b"\n").await?;
        }

        Ok(())
    }

    /// Gets the line-column location of the span in its file.
    ///
    /// ## Panics
    ///
    /// This function will panic if `span` refers to a span in a file not registered with this reporter, e.g.
    ///
    /// ```should_panic
    /// # use nurse::prelude::*;
    /// let mut reporter1 = TerminalReporter::default();
    /// let reporter2 = TerminalReporter::default();
    ///
    /// let key = reporter1.register_file("example.txt", r#""hello world""#);
    /// let span = Span::new(key, 0..1);
    /// // Should panic!
    /// reporter2.location(span);
    /// ```
    ///
    pub async fn location(&self, span: Span) -> Location {
        let lookups = self.lookups.lock().await;

        let lookup = &lookups
            .get(span.lookup())
            .expect("span should refer to an already registered file")
            .1;

        let (line, column) = lookup.line_col(span.start());
        Location { line, column }
    }

    /// Returns a single character-wide span at the end of the file referred to by `key`.
    ///
    /// This is useful raising errors if you expect a token,
    /// but instead find the end of a file.
    ///
    /// ## Panics
    ///
    /// This function will panic if `key` refers to a file not registered with this reporter, e.g.
    ///
    /// ```should_panic
    /// # use nurse::TerminalReporter;
    /// let mut reporter1 = TerminalReporter::default();
    /// let reporter2 = TerminalReporter::default();
    ///
    /// let key = reporter1.register_file("example.txt", r#""hello world""#);
    /// // Should panic!
    /// reporter2.eof_span(key);
    /// ```
    ///
    /// ## Example
    ///
    /// ```rust
    /// # use nurse::prelude::*;
    /// # let mut reporter = TerminalReporter::default();
    /// let key = reporter.register_file("example.txt", "3 + ");
    /// let eof_span = reporter.eof_span(key);
    ///
    /// reporter.report(error!(eof_span, "expected token, found EOF"));
    /// ```
    pub async fn eof_span(&self, key: LookupKey) -> Span {
        let lookups = self.lookups.lock().await;
        let (_, lookup) = lookups.get(key)
            .expect("key should refer to an already registered file");

        let eof = lookup.file_len();

        Span {
            start: eof,
            end: eof + 1,
            lookup: key,
        }
    }

    async fn pointer(
        &self,
        span: Span,
        note: Option<(&str, Span)>,
        arrow_color: Color,
    ) -> (String, usize) {
        let lookups = self.lookups.lock().await;
        let (file, lookup) = lookups.get(span.lookup()).expect("span should ");

        let lines = lookup.lines(span.start()..span.end());
        let line_n = lines.start + 1;
        let col_n = lookup.col_from_line(lines.start, span.start()) + 1;

        if lines.len() > 1 {
            let start = lookup.line(lines.start).trim_end();
            let end = lookup.line(lines.end - 1).trim_end();

            let end_col = lookup.col_from_line(lines.end - 1, span.end());

            let offset = ((lines.end + 1).ilog10()).max(2) as usize + 2;

            (
                format!(
                    "\
                    {arrow:>arr_space$} [{name}:{line_n}:{col_n}]\n\
                    {cap:>width$}\n\
                    {start_n}   {start}\n\
                    {cap:>width$} {start_pointer}\n\
                    {dot_n}  {pipe}\n\
                    {end_n} {pipe} {end}\n\
                    {cap:>width$} {end_pointer}\n\
                    ",
                    arrow = "——>".bright_blue().bold(),
                    arr_space = offset + 2,
                    name = file.bold().bright_cyan().underline(),
                    cap = "┃".bright_blue().bold(),
                    width = offset + 1,
                    start_n = format!("{:<offset$}┃", lines.start + 1)
                        .bright_blue()
                        .bold(),
                    dot_n = format!("{:<offset$}", "...").bright_blue().bold(),
                    end_n = format!("{:<offset$}┃", lines.end + 1).bright_blue().bold(),
                    start_pointer = format!(
                        "╭─{blank:·>start$}{blank:—>length$}",
                        blank = "",
                        start = col_n - 1,
                        length = start.len() - col_n + 1,
                    )
                    .color(arrow_color),
                    end_pointer = format!("╰─{blank:─>length$}", blank = "", length = end_col)
                        .color(arrow_color),
                    pipe = "│".color(arrow_color)
                ),
                offset,
            )
        } else {
            let line = lookup.line(lines.start).trim_end();
            let offset = (lines.start + 1).ilog10() as usize + 2;

            (
                format!(
                    "\
                    {arrow:>arr_space$} [{name}:{line_n}:{col_n}]\n\
                    {cap:>width$}\n\
                    {n}{cap} {line}\n\
                    {cap:>width$} {pointer}\
                    ",
                    arrow = "——>".bright_blue().bold(),
                    name = file.bold().bright_cyan().underline(),
                    cap = "┃".bright_blue().bold(),
                    width = offset + 1,
                    arr_space = offset + 2,
                    n = format!("{line_n:<offset$}").bright_blue().bold(),
                    pointer = format!(
                        "{blank:>start$}{blank:‾>length$}",
                        blank = "",
                        start = col_n - 1,
                        length = span.end() - span.start(),
                    )
                    .bold()
                    .color(arrow_color),
                ),
                offset,
            )
        }
    }
}

impl Default for TerminalReporter<std::io::Stdout> {
    fn default() -> Self {
        TerminalReporter {
            diagnostics: Vec::new().into(),
            lookups: SlotMap::with_key().into(),
            filter: LevelFilter::Debug,
            emitter: new_emitter(std::io::stdout()),
        }
    }
}
