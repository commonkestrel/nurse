use std::{
    borrow, fmt::{Debug, Formatter}, ops::{self, Range}, sync::Arc
};

use colored::{Color, Colorize};

use crate::lookup::{Location, Lookup};

#[derive(PartialEq, Clone)]
pub struct Spanned<T> {
    pub inner: T,
    pub span: Span,
}

impl<T> Spanned<T> {
    #[inline]
    pub fn new(value: T, span: Span) -> Spanned<T> {
        Spanned { inner: value, span }
    }

    #[inline]
    pub fn inner(&self) -> &T {
        &self.inner
    }

    #[inline]
    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.inner
    }

    #[inline]
    pub fn into_inner(self) -> T {
        self.inner
    }

    #[inline]
    pub fn span(&self) -> &Span {
        &self.span
    }

    #[inline]
    pub fn into_span(self) -> Span {
        self.span
    }

    #[inline]
    pub fn deconstruct(self) -> (T, Span) {
        (self.inner, self.span)
    }

    pub fn map<M, O>(self, map: M) -> Spanned<O>
    where
        M: FnOnce(T) -> O,
    {
        Spanned::new(map(self.inner), self.span)
    }
}

impl<T: Debug> Debug for Spanned<T> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "{:?}", self.inner())
    }
}

impl<T> ops::Deref for Spanned<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.inner()
    }
}

impl<T> ops::DerefMut for Spanned<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner_mut()
    }
}

impl<T> borrow::Borrow<T> for Spanned<T> {
    #[inline]
    fn borrow(&self) -> &T {
        self.inner()
    }
}

impl<T> borrow::BorrowMut<T> for Spanned<T> {
    #[inline]
    fn borrow_mut(&mut self) -> &mut T {
        self.inner_mut()
    }
}

#[derive(Clone, PartialEq)]
pub struct Span {
    source_name: Arc<String>,
    lookup: Arc<Lookup>,
    location: Range<usize>,
}

impl Debug for Span {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Span")
            .field("source", &self.source_name)
            .field("location", &self.location)
            .finish()
    }
}

impl Span {
    pub fn new(source_name: Arc<String>, lookup: Arc<Lookup>, location: Range<usize>) -> Self {
        Span {
            source_name,
            lookup,
            location,
        }
    }

    pub fn start(&self) -> usize {
        return self.location.start;
    }

    pub fn end(&self) -> usize {
        return self.location.end;
    }

    pub fn with_location(mut self, location: Range<usize>) -> Self {
        self.location = location;
        self
    }

    pub fn source_name(&self) -> Arc<String> {
        self.source_name.clone()
    }

    pub fn lookup(&self) -> Arc<Lookup> {
        self.lookup.clone()
    }

    pub fn to(&self, other: &Span) -> Span {
        debug_assert_eq!(self.source_name, other.source_name);
        debug_assert_eq!(self.lookup, other.lookup);

        Span {
            source_name: self.source_name.clone(),
            lookup: self.lookup.clone(),
            location: self.location.start.min(other.location.start)
                ..self.location.end.max(other.location.end),
        }
    }

    pub fn location(&self) -> Location {
        let (line, column) = self.lookup.line_col(self.location.start);
        Location { line, column }
    }

    pub fn pointer(&self, arrow_color: Color) -> (String, usize) {
        let lines = self.lookup.lines(self.location.clone());
        let line_n = lines.start + 1;
        let col_n = self.lookup.col_from_line(lines.start, self.location.start) + 1;

        if lines.len() > 1 {
            todo!()
        } else {
            let line = self.lookup.line(lines.start).trim_end();
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
                    name = self.source_name,
                    cap = "|".bright_blue().bold(),
                    width = offset + 1,
                    arr_space = offset + 2,
                    n = format!("{line_n:<offset$}|").bright_blue().bold(),
                    pointer = format!(
                        "{blank:>start$}{blank:^>length$}",
                        blank = "",
                        start = col_n - 1,
                        length = self.location.end - self.location.start,
                    )
                    .color(arrow_color),
                ),
                offset,
            )
        }
    }
}
