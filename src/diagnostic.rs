use crate::span::Span;

use colored::{Color, ColoredString, Colorize};

#[must_use = "Diagnostics should either be emitted or reported!"]
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub(crate) level: Level,
    pub(crate) message: String,
    pub(crate) note: Option<Note>,
    pub(crate) span: Option<Span>,
}

impl Diagnostic {
    pub fn error<S: Into<String>>(message: S) -> Self {
        Diagnostic {
            level: Level::Error,
            message: message.into(),
            note: None,
            span: None,
        }
    }

    pub fn spanned_error<M: Into<String>>(span: Span, message: M) -> Self {
        Diagnostic {
            level: Level::Error,
            message: message.into(),
            note: None,
            span: Some(span),
        }
    }

    pub fn warning<S: Into<String>>(message: S) -> Self {
        Diagnostic {
            level: Level::Warn,
            message: message.into(),
            note: None,
            span: None,
        }
    }

    pub fn spanned_warning<M: Into<String>>(span: Span, message: M) -> Self {
        Diagnostic {
            level: Level::Warn,
            message: message.into(),
            note: None,
            span: Some(span),
        }
    }

    pub fn info<S: Into<String>>(message: S) -> Self {
        Diagnostic {
            level: Level::Info,
            message: message.into(),
            note: None,
            span: None,
        }
    }

    pub fn spanned_info<M: Into<String>>(span: Span, message: M) -> Self {
        Diagnostic {
            level: Level::Info,
            message: message.into(),
            note: None,
            span: Some(span),
        }
    }

    pub fn hint<S: Into<String>>(message: S) -> Self {
        Diagnostic {
            level: Level::Hint,
            message: message.into(),
            note: None,
            span: None,
        }
    }

    pub fn spanned_hint<M: Into<String>>(span: Span, message: M) -> Self {
        Diagnostic {
            level: Level::Hint,
            message: message.into(),
            note: None,
            span: Some(span),
        }
    }

    pub fn set_span(&mut self, span: Option<Span>) {
        self.span = span;
    }

    #[inline]
    pub fn with_span(mut self, span: Option<Span>) -> Self {
        self.set_span(span);
        self
    }

    pub fn set_message<S: Into<String>>(&mut self, message: S) {
        self.message = message.into();
    }

    #[inline]
    pub fn with_message<S: Into<String>>(mut self, message: S) -> Self {
        self.set_message(message);
        self
    }

    pub fn set_note<S: Into<Note>>(&mut self, note: S) {
        self.note = Some(note.into());
    }

    #[inline]
    pub fn with_note<S: Into<Note>>(mut self, note: S) -> Self {
        self.set_note(note);
        self
    }

    #[inline]
    pub fn set_spanned_note<S: Into<String>>(&mut self, note: S, span: Span) {
        self.set_note(Note {
            value: note.into(),
            span: Some(span),
        });
    }

    #[inline]
    pub fn with_spanned_note<S: Into<String>>(mut self, note: S, span: Span) -> Self {
        self.set_spanned_note(note, span);
        self
    }

    pub fn clear_note(&mut self) {
        self.note = None;
    }

    pub fn without_note(mut self) -> Self {
        self.clear_note();
        self
    }

    #[inline]
    pub fn level(&self) -> Level {
        self.level
    }

    #[inline]
    pub fn is_error(&self) -> bool {
        self.level == Level::Error
    }

    pub(crate) fn format_message(&self) -> ColoredString {
        let title = self.level.title();
        let color = self.level.color();

        format!("{}: {}", title.color(color), self.message).bold()
    }
}

impl PartialEq for Diagnostic {
    fn eq(&self, other: &Self) -> bool {
        self.message == other.message && self.level == other.level
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Note {
    pub(crate) value: String,
    pub(crate) span: Option<Span>,
}

impl Into<Note> for String {
    fn into(self) -> Note {
        Note {
            value: self,
            span: None,
        }
    }
}

impl Into<Note> for &str {
    fn into(self) -> Note {
        Note {
            value: self.to_owned(),
            span: None,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Level {
    Error,
    Warn,
    Info,
    Hint,
}

impl Level {
    pub fn title(&self) -> &'static str {
        match self {
            Level::Error => "error",
            Level::Warn => "warn",
            Level::Info => "info",
            Level::Hint => "hint",
        }
    }

    pub fn color(&self) -> Color {
        match self {
            Level::Error => Color::Red,
            Level::Warn => Color::Yellow,
            Level::Info => Color::White,
            Level::Hint => Color::Cyan,
        }
    }
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => ($crate::diagnostic::Diagnostic::error(::std::format!($($arg)*)))
}

#[macro_export]
macro_rules! spanned_error {
    ($span:expr, $($arg:tt)*) => ($crate::diagnostic::Diagnostic::spanned_error($span, ::std::format!($($arg)*)))
}

#[macro_export]
macro_rules! warning {
    ($($arg:tt)*) => ($crate::diagnostic::Diagnostic::warning(::std::format!($($arg)*)))
}

#[macro_export]
macro_rules! spanned_warning {
    ($span:expr, $($arg:tt)*) => ($crate::diagnostic::Diagnostic::spanned_warning($span, ::std::format!($($arg)*)))
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => ($crate::diagnostic::Diagnostic::info(::std::format!($($arg)*)))
}

#[macro_export]
macro_rules! spanned_info {
    ($span:expr, $($arg:tt)*) => ($crate::diagnostic::Diagnostic::spanned_info($span, ::std::format!($($arg)*)))
}

#[macro_export]
macro_rules! hint {
    ($($arg:tt)*) => ($crate::diagnostic::Diagnostic::debug(::std::format!($($arg)*)))
}

#[macro_export]
macro_rules! spanned_hint {
    ($span:expr, $($arg:tt)*) => ($crate::diagnostic::Diagnostic::spanned_debug($span, ::std::format!($($arg)*)))
}
