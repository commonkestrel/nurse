use std::{
    borrow,
    fmt::{Debug, Formatter},
    ops::{self, Range},
};

use crate::reporter::LookupKey;

/// A token associated with a [`Span`].
#[derive(PartialEq, Clone)]
pub struct Spanned<T> {
    inner: T,
    span: Span,
}

impl<T> Spanned<T> {
    /// Creates a new [`Spanned`] with the provided `value` and `span`.
    pub fn new(value: T, span: Span) -> Spanned<T> {
        Spanned { inner: value, span }
    }

    /// Returns a reference to the inner value.
    pub fn inner(&self) -> &T {
        &self.inner
    }

    /// Returns a mutable reference to the inner value.
    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.inner
    }

    /// Returns ownership of the inner value.
    pub fn into_inner(self) -> T {
        self.inner
    }

    /// Returns the span of the inner value.
    #[inline]
    pub fn span(&self) -> Span {
        self.span
    }

    /// Returns ownership of the inner value and span.
    pub fn deconstruct(self) -> (T, Span) {
        (self.inner, self.span)
    }

    /// Maps the inner value to the output of the input `map`
    ///
    /// ## Example
    ///
    /// ```rust
    /// # use nurse::prelude::*;
    /// # let mut reporter = TerminalReporter::default();
    /// # let key = reporter.register_file("example.txt", "123");
    /// # let span = Span::new(key, 0..3);
    /// let mut token = Spanned::new(123, span);
    /// token = token.map(|tok| tok * 2);
    /// 
    /// assert_eq!(token.into_inner(), 246);
    /// ```
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

    fn deref(&self) -> &Self::Target {
        self.inner()
    }
}

impl<T> ops::DerefMut for Spanned<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner_mut()
    }
}

impl<T> borrow::Borrow<T> for Spanned<T> {
    fn borrow(&self) -> &T {
        self.inner()
    }
}

impl<T> borrow::BorrowMut<T> for Spanned<T> {
    fn borrow_mut(&mut self) -> &mut T {
        self.inner_mut()
    }
}

#[cfg(not(feature = "terminal"))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Span {
    pub(crate) start: usize,
    pub(crate) end: usize,
}

/// A range of characters within a file.
#[cfg(feature = "terminal")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Span {
    pub(crate) lookup: LookupKey,
    // We split the input range into a start and end,
    // since `std::ops::Range` is not `Copy`
    pub(crate) start: usize,
    pub(crate) end: usize,
}

impl Span {
    /// Gets the start of the span's character range
    pub fn start(&self) -> usize {
        return self.start;
    }

    /// Gets the end of the span's character range
    pub fn end(&self) -> usize {
        return self.end;
    }

    /// Gets the span's character range
    pub fn range(&self) -> Range<usize> {
        self.start..self.end
    }
}

#[cfg(feature = "terminal")]
impl Span {
    /// Creates a new `Span` with the given lookup key and location.
    ///
    /// `location` should be a range of character indices contained within the file `lookup` refers to.
    pub fn new(lookup: LookupKey, location: Range<usize>) -> Span {
        Span {
            lookup,
            start: location.start,
            end: location.end,
        }
    }

    /// Gets the lookup of the span.
    pub fn lookup(&self) -> LookupKey {
        self.lookup
    }

    /// Creates a new span containing both input spans.
    ///
    /// ## Example
    ///
    /// ```rust
    /// # use nurse::prelude::*;
    /// # let mut reporter = TerminalReporter::default();
    /// let lookup = reporter.register_file("test.txt", "foo bar");
    ///
    /// let foo_span = Span::new(lookup, 0..3);
    /// let bar_span = Span::new(lookup, 4..7);
    /// let span = foo_span.to(bar_span);
    ///
    /// assert_eq!(span.lookup(), lookup);
    /// assert_eq!(span.range(), 0..7);
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if the lookup keys are not equal.
    pub fn to(&self, other: Span) -> Span {
        assert_eq!(self.lookup, other.lookup);

        Span {
            lookup: other.lookup,
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }
}

#[cfg(not(feature = "terminal"))]
impl Span {
    // Creates a new `Span` with the given location.
    ///
    /// `location` should be a range of character indices contained within a file.
    pub fn new(location: Range<usize>) -> Span {
        Span {
            start: location.start,
            end: location.end,
        }
    }

    /// Creates a new span containing both input spans.
    ///
    /// ## Example
    ///
    /// ```rust
    /// # use nurse::prelude::*;
    ///
    /// let foo_span = Span::new(0..3);
    /// let bar_span = Span::new(4..7);
    /// let span = foo_span.to(bar_span);
    ///
    /// assert_eq!(span.range(), 0..7);
    /// ```
    pub fn to(&self, other: Span) -> Span {
        Span {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }
}

/// The trait allowing for multiple different types to be passed into the
/// [`error`](crate::error), [`warning`](crate::warning), etc. macros to indicte span.
pub trait MaybeSpanned {
    /// Gets the span of the structure if a span exists,
    /// otherwise returns [`None`].
    #[inline]
    fn get_span(&self) -> Option<Span> {
        None
    }
}

impl MaybeSpanned for Span {
    #[inline]
    fn get_span(&self) -> Option<Span> {
        return Some(*self);
    }
}

impl<T> MaybeSpanned for Spanned<T> {
    #[inline]
    fn get_span(&self) -> Option<Span> {
        return Some(self.span);
    }
}
