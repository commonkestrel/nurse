use std::{
    borrow,
    fmt::{Debug, Formatter},
    ops::{self, Range},
};

use crate::reporter::LookupKey;

#[derive(PartialEq, Clone)]
pub struct Spanned<T> {
    inner: T,
    span: Span,
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
    pub fn span(&self) -> Span {
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

#[cfg(not(feature = "serial"))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Span {
    pub(crate) start: usize,
    pub(crate) end: usize,
}

#[cfg(feature = "serial")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Span {
    pub(crate) lookup: LookupKey,
    pub(crate) start: usize,
    pub(crate) end: usize,
}

impl Span {
    pub fn new(lookup: LookupKey, location: Range<usize>) -> Self {
        Span {
            lookup,
            start: location.start,
            end: location.end,
        }
    }

    pub fn start(&self) -> usize {
        return self.start;
    }

    pub fn end(&self) -> usize {
        return self.end;
    }
}

#[cfg(feature = "serial")]
impl Span {
    pub fn with_location(mut self, location: Range<usize>) -> Self {
        self.start = location.start;
        self.end = location.end;
        self
    }

    pub fn lookup_key(&self) -> LookupKey {
        self.lookup
    }

    pub fn to(&self, other: Span) -> Span {
        assert_eq!(self.lookup, other.lookup);

        Span {
            lookup: other.lookup,
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }
}

#[cfg(not(feature = "serial"))]
impl Span {
    pub fn to(&self, other: Span) -> Span {
        Span {
            start: self.start.min(other.start),
            end: self.end.max(other.max),
        }
    }
}

pub trait MaybeSpanned {
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
