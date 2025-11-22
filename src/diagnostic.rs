use std::borrow::Cow;

use crate::span::Span;
use concat_idents::concat_idents;

use colored::{Color, ColoredString, Colorize};

/// A diagnostic message ready to be output.
#[must_use = "Diagnostics should either be emitted or reported!"]
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub(crate) level: Level,
    pub(crate) message: String,
    pub(crate) note: Option<Note>,
    pub(crate) span: Option<Span>,
}

macro_rules! diagnostic_level {
    (($d:tt) $name:ident, $variant:ident) => {
        #[doc = concat!("Creates a [`Diagnostic`] with a level of [`Level::", stringify!($variant), "`] with a formatted message.")]
        ///
        /// In addition, it can optionally insert a span from a [`Span`] or [`Spanned`](crate::span::Spanned) item.
        /// Formatting uses the same syntax as [`format!`](std::format).
        ///
        /// ## Simple diagnostic
        ///
        /// ```rust
        /// use nurse::prelude::*;
        /// use nurse::diagnostic::Level;
        ///
        /// # fn main() {
        /// let message = "Maybe a missing parenthesis?";
        #[doc = concat!("let diagnostic: Diagnostic = ", stringify!($name), r#"!("Something's wrong! {message}");"#)]
        ///
        #[doc = concat!("assert_eq!(diagnostic.level(), Level::", stringify!($variant), ");")]
        /// assert_eq!(diagnostic.message(), format!("Something's wrong! {message}"));
        /// # }
        /// ```
        ///
        /// ## Spanned diagnostic
        ///
        /// ```rust
        /// use nurse::prelude::*;
        /// use nurse::diagnostic::Level;
        ///
        /// # fn main() {
        /// # let mut reporter = TerminalReporter::default();
        /// # let file = reporter.register_file("example.txt", "266");
        /// # let span = Span::new(file, 0..3);
        /// let token: Spanned<usize> = Spanned::new(266, span);
        #[doc = concat!("let diagnostic = ", stringify!($name), r#"!(token, "Integer too large for `u8` ({} > 255)", token.inner());"#)]
        ///
        #[doc = concat!("assert_eq!(diagnostic.level(), Level::", stringify!($variant), ");")]
        /// assert_eq!(
        ///     diagnostic.message(),
        ///     format!("Integer too large for `u8` ({} > 255)", token.inner())
        /// );
        /// assert_eq!(diagnostic.span(), Some(token.span()));
        /// # }
        /// ```
        #[macro_export]
        macro_rules! $name {
            ($fmt:literal $d($arg:tt)*) => ($crate::Diagnostic::$name(::std::format!($fmt $d($arg)*)));
            ($span:expr, $fmt:literal $d($arg:tt)*) => {{
                use $crate::MaybeSpanned;
                $crate::Diagnostic::$name(::std::format!($fmt $d($arg)*)).with_span($span.get_span())
            }};
        }

        concat_idents!{awawa = _, $name {
            #[doc(hidden)]
            pub use $name as awawa;
        }}
    };
    ($name:ident, $variant:ident) => {
        impl Diagnostic {
            #[doc = concat!("Creates a new [`Diagnostic`] with a level of [`Level::", stringify!($variant), "`] with the input message.")]
            pub fn $name<S: Into<String>>(message: S) -> Diagnostic {
                Diagnostic {
                    level: Level::$variant,
                    message: message.into(),
                    note: None,
                    span: None,
                }
            }

            concat_idents!{spanned = spanned_, $name {
                #[doc = concat!("Creates a new [`Diagnostic`] with a level of [`Level::", stringify!($variant), "`] with the input message and span.")]
                pub fn spanned<M: Into<String>>(span: Span, message: M) -> Diagnostic {
                    Diagnostic {
                        level: Level::$variant,
                        message: message.into(),
                        note: None,
                        span: Some(span),
                    }
                }
            }}
        }

        diagnostic_level!(($) $name, $variant);
    }
}
diagnostic_level!(error, Error);
diagnostic_level!(warning, Warn);
diagnostic_level!(info, Info);
diagnostic_level!(debug, Debug);

impl Diagnostic {
    /// Gets the span of the diagnostic if it exists,
    /// returning `None` if there is not.
    #[inline]
    pub fn span(&self) -> Option<Span> {
        self.span
    }

    /// Sets the span of the diagnostic
    #[inline]
    pub fn set_span(&mut self, span: Option<Span>) -> &mut Diagnostic {
        self.span = span;
        self
    }

    /// Sets the span of the diagnostic
    #[inline]
    pub fn with_span(mut self, span: Option<Span>) -> Diagnostic {
        self.set_span(span);
        self
    }

    /// Gets the message of the diagnostic
    #[inline]
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Sets the message of the diagnostic
    #[inline]
    pub fn set_message<S: Into<String>>(&mut self, message: S) -> &mut Diagnostic {
        self.message = message.into();
        self
    }

    /// Sets the message of the diagnostic
    #[inline]
    pub fn with_message<S: Into<String>>(mut self, message: S) -> Diagnostic {
        self.set_message(message);
        self
    }

    /// Sets the optional note of the diagnostic.
    pub fn set_note<S: Into<Note>>(&mut self, note: S) -> &mut Diagnostic {
        self.note = Some(note.into());
        self
    }

    /// Sets the optional note of the diagnostic.
    #[inline]
    pub fn with_note<S: Into<Note>>(mut self, note: S) -> Diagnostic {
        self.set_note(note);
        self
    }

    /// Sets the optional note of the diagnostic with the given span.
    #[inline]
    pub fn set_spanned_note<S: Into<String>>(&mut self, note: S, span: Span) -> &mut Diagnostic {
        self.set_note(Note {
            value: note.into(),
            span: Some(span),
        });
        self
    }

    /// Sets the optional note of the diagnostic with the given span.
    #[inline]
    pub fn with_spanned_note<S: Into<String>>(mut self, note: S, span: Span) -> Diagnostic {
        self.set_spanned_note(note, span);
        self
    }

    /// Gets the [`Level`] associated with the diagnostic.
    #[inline]
    pub fn level(&self) -> Level {
        self.level
    }

    /// Checks if the level associated with the diagnostic is [`Level::Error`].
    #[inline]
    pub fn is_error(&self) -> bool {
        self.level == Level::Error
    }

    pub(crate) fn format_message(&self) -> ColoredString {
        let title = self.level.title();
        let color = self.level.color();

        let formatted = format!("{}: {}", title.color(color).bold(), self.message);
        if self.span.is_some() {
            formatted.bold()
        } else {
            formatted.into()
        }
    }
}

impl PartialEq for Diagnostic {
    fn eq(&self, other: &Diagnostic) -> bool {
        self.message == other.message && self.level == other.level
    }
}

/// An optional note to be associated with a [`Diagnostic`].
#[derive(Debug, Clone)]
pub struct Note {
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

impl Into<Note> for Cow<'_, str> {
    fn into(self) -> Note {
        Note {
            value: self.into_owned(),
            span: None,
        }
    }
}

/// The level of a diagnostic.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Level {
    /// The "error" level.
    ///
    /// Designates fatal compilation errors.
    Error,
    /// The "warn" level.
    ///
    /// Designates potential problems.
    Warn,
    /// The "info" level.
    ///
    /// Designates helpful information.
    Info,
    /// The "debug" level.
    ///
    /// Designates low-priority debugging information,
    /// often for program authors.
    Debug,
}

impl Level {
    pub(crate) fn title(&self) -> &'static str {
        match self {
            Level::Error => "error",
            Level::Warn => "warning",
            Level::Info => "info",
            Level::Debug => "debug",
        }
    }

    pub(crate) fn color(&self) -> Color {
        match self {
            Level::Error => Color::Red,
            Level::Warn => Color::Yellow,
            Level::Info => Color::Cyan,
            Level::Debug => Color::BrightMagenta,
        }
    }
}

/// The filter for which level diagnostics are allowed.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum LevelFilter {
    /// Disables all diagnostics
    Off,
    /// Disables all but [`Error`](Level::Error) level diagnostics.
    Error,
    /// Disables all but [`Error`](Level::Error) and [`Warn`](Level::Warn) level diagnostics.
    Warn,
    /// Enables all but [`Debug`](Level::Debug) level diagnostics.
    Info,
    /// Enables all diagnostics.
    Debug,
}

impl LevelFilter {
    pub(crate) fn passes(&self, level: Level) -> bool {
        match (self, level) {
            (LevelFilter::Error, Level::Error) => true,
            (LevelFilter::Warn, Level::Error | Level::Warn) => true,
            (LevelFilter::Info, Level::Error | Level::Warn | Level::Info) => true,
            (LevelFilter::Debug, _) => true,
            _ => false,
        }
    }
}
