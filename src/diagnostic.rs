use crate::span::Span;
use concat_idents::concat_idents;

use colored::{Color, Colorize};

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
        /// # let mut reporter = Reporter::new();
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
            ($fmt:literal $d($arg:tt)*) => ($crate::diagnostic::Diagnostic::$name(::std::format!($fmt $d($arg)*)));
            ($span:expr, $fmt:literal $d($arg:tt)*) => {{
                use $crate::span::MaybeSpanned;
                $crate::diagnostic::Diagnostic::$name(::std::format!($fmt $d($arg)*)).with_span($span.get_span())
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
diagnostic_level!(hint, Hint);
diagnostic_level!(debug, Debug);

impl Diagnostic {
    pub fn span(&self) -> Option<Span> {
        self.span
    }

    pub fn set_span(&mut self, span: Option<Span>) {
        self.span = span;
    }

    #[inline]
    pub fn with_span(mut self, span: Option<Span>) -> Self {
        self.set_span(span);
        self
    }
    
    pub fn message(&self) -> &str {
        &self.message
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

    pub(crate) fn format_message(&self) -> String {
        let title = self.level.title();
        let color = self.level.color();

        format!("{}: {}", title.color(color).bold(), self.message)
    }
}

impl PartialEq for Diagnostic {
    fn eq(&self, other: &Self) -> bool {
        self.message == other.message && self.level == other.level
    }
}

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

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Level {
    Error,
    Warn,
    Info,
    Hint,
    Debug,
}

impl Level {
    pub fn title(&self) -> &'static str {
        match self {
            Level::Error => "error",
            Level::Warn => "warn",
            Level::Info => "info",
            Level::Hint => "hint",
            Level::Debug => "debug",
        }
    }

    pub fn color(&self) -> Color {
        match self {
            Level::Error => Color::Red,
            Level::Warn => Color::Yellow,
            Level::Info => Color::White,
            Level::Hint => Color::Cyan,
            Level::Debug => Color::BrightMagenta
        }
    }
}
