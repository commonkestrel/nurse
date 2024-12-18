use std::{collections::HashMap, io};

use colored::{Color, Colorize};
use slotmap::SlotMap;

use crate::{
    diagnostic::{Diagnostic, Level},
    lookup::{Location, Lookup},
    span::Span,
};

use super::{LookupKey, Reporter};

#[derive(Debug, Clone)]
pub struct SerialReporter {
    diagnostics: Vec<Diagnostic>,
    lookups: SlotMap<LookupKey, (String, Lookup)>,
}

impl SerialReporter {
    pub fn new() -> SerialReporter {
        Self::default()
    }

    pub fn register_file<N: ToString>(&mut self, name: N, contents: String) -> LookupKey {
        self.lookups
            .insert((name.to_string(), Lookup::new(contents)))
    }

    pub fn emit<E: MaybeTerminal>(&self, emitter: &mut E, diagnostic: Diagnostic) {
        if emitter.is_terminal() {
            Self::emit_fancy(&self.lookups, emitter, diagnostic);
        } else {
            Self::raw_emit(emitter, diagnostic);
        }
    }

    #[cfg(any(feature = "async-std", feature = "tokio"))]
    pub fn emit_async<E: MaybeAsyncTerminal>(&self, emitter: &mut E, diagnostic: Diagnostic) {
        todo!()
    }

    pub fn emit_all<E: MaybeTerminal>(&mut self, emitter: &mut E) {
        for diagnostic in self.diagnostics.drain(..) {
            if emitter.is_terminal() {
                Self::emit_fancy(&self.lookups, emitter, diagnostic);
            } else {
                Self::raw_emit(emitter, diagnostic);
            }
            writeln!(emitter);
        }
    }

    fn raw_emit<E: io::Write>(emitter: &mut E, diagnostic: Diagnostic) {
        let title = diagnostic.level.title();
        writeln!(emitter, "{title}: {}", diagnostic.message);
    }

    fn emit_fancy<E: io::Write>(lookups: &SlotMap<LookupKey, (String, Lookup)>, emitter: &mut E, diagnostic: Diagnostic) {
        let mut note_offset = diagnostic.level.title().len() + 1;
        let message = diagnostic.format_message();
        println!("{message}");

        if let Some(span) = diagnostic.span {
            let (pointer, offset) = Self::pointer(lookups, span, diagnostic.level.color());
            note_offset = offset + 1;
            println!("{pointer}");
        }

        if let Some(note) = diagnostic.note {
            writeln!(
                emitter,
                "{:>note_offset$} {}: {}",
                "=".bright_blue().bold(),
                "note".bold(),
                note.value
            );
        }
    }

    pub fn pointer(lookups: &SlotMap<LookupKey, (String, Lookup)>, span: Span, arrow_color: Color) -> (String, usize) {
        let (file, lookup) = lookups.get(span.lookup_key()).expect("span should ");

        let lines = lookup.lines(span.start()..span.end());
        let line_n = lines.start + 1;
        let col_n = lookup.col_from_line(lines.start, span.start()) + 1;

        if lines.len() > 1 {
            todo!()
        } else {
            let line = lookup.line(lines.start).trim_end();
            let offset = (lines.start + 1).ilog10() as usize + 2;

            (
                format!(
                    "\
                {arrow:>arr_space$} {name}:{line_n}:{col_n}\n\
                {cap:>width$}\n\
                {n} {line}\n\
                {cap:>width$} {pointer}\
                ",
                    arrow = "-->".bright_blue().bold(),
                    name = file,
                    cap = "|".bright_blue().bold(),
                    width = offset + 1,
                    arr_space = offset + 2,
                    n = format!("{line_n:<offset$}|").bright_blue().bold(),
                    pointer = format!(
                        "{blank:>start$}{blank:^>length$}",
                        blank = "",
                        start = col_n - 1,
                        length = span.end() - span.start(),
                    )
                    .color(arrow_color),
                ),
                offset,
            )
        }
    }

    pub fn location(&self, span: Span) -> Location {
        let lookup = &self
            .lookups
            .get(span.lookup_key())
            .expect("span should refer to an already registered file")
            .1;
        
        let (line, column) = lookup.line_col(span.start());
        Location { line, column }
    }
}

impl Reporter for SerialReporter {
    fn report(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    fn report_all(&mut self, mut diagnostics: Vec<Diagnostic>) {
        self.diagnostics.append(&mut diagnostics);
    }

    fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|diagnostic| diagnostic.is_error())
    }

    fn is_empty(&self) -> bool {
        self.diagnostics.is_empty()
    }

    fn eof_span(&self, key: LookupKey) -> Span {
        let (_, lookup) = self.lookups.get(key).expect("key should refer to an already registered file");

        let eof = lookup.file_len();
        
        Span {
            start: eof,
            end: eof+1,
            lookup: key,
        }
    }
}

impl Default for SerialReporter {
    fn default() -> Self {
        SerialReporter {
            diagnostics: Vec::new(),
            lookups: SlotMap::with_key(),
        }
    }
}

pub trait MaybeTerminal: io::Write + io::IsTerminal {}
impl<T: io::Write + io::IsTerminal> MaybeTerminal for T {}

#[cfg(feature = "async-std")]
pub trait MaybeAsyncTerminal: async_std::io::WriteExt + io::IsTerminal {}
#[cfg(feature = "async-std")]
impl<T: async_std::io::WriteExt + io::IsTerminal> MaybeAsyncTerminal for T {}

#[cfg(feature = "tokio")]
pub trait MaybeAsyncTerminal: tokio::io::AsyncWriteExt + io::IsTerminal {}
#[cfg(feature = "tokio")]
impl<T: tokio::io::AsyncWriteExt + io::IsTerminal> MaybeAsyncTerminal for T {}
