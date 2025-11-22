//! Containing utilities surrounding [`Lookup`].

use std::{cmp::Ordering, ops::Range};

/// Internal file lookup-table used in [`Reporter`](crate::reporter::Reporter)s
/// to locate lines, columns, and text.
#[derive(Debug, Clone, PartialEq)]
pub struct Lookup {
    source: String,
    heads: Box<[usize]>,
}

impl Lookup {
    pub fn new(source: String) -> Self {
        // Replace tabs with spaces in order to keep character spacing the same
        let source = source.replace('\t', " ");

        let heads = std::iter::once(0)
            .chain(
                source
                    .char_indices()
                    .filter_map(|(i, c)| if c == '\n' { Some(i + 1) } else { None }),
            )
            .collect();

        Lookup { source, heads }
    }

    pub fn line_n(&self, index: usize) -> usize {
        match self.heads.binary_search(&index) {
            Ok(line) => line,
            Err(insert) => (insert - 1),
        }
    }

    #[inline]
    pub fn line_col(&self, index: usize) -> (usize, usize) {
        let line = self.line_n(index);
        let col = self.col_from_line(line, index);

        (line, col)
    }

    #[inline]
    pub fn col_from_line(&self, line: usize, index: usize) -> usize {
        index - self.heads[line]
    }

    pub fn multiline(&self, range: Range<usize>) -> bool {
        let starting_line = self.line_n(range.start);
        let next_start = self.heads[starting_line + 1];

        range.end <= next_start
    }

    pub fn line(&self, index: usize) -> &str {
        let range = self.heads[index]..(*self.heads.get(index + 1).unwrap_or(&self.source.len()));
        &self.source[range]
    }

    pub fn lines(&self, span: Range<usize>) -> Range<usize> {
        let start_line = self.line_n(span.start);
        let next_start = *self.heads.get(start_line + 1).unwrap_or(&self.source.len());

        if span.end <= next_start {
            // Check if the span ends on the same line
            start_line..start_line + 1
        } else {
            // Otherwise perform a binary search through the rest of the lines.
            match self.heads[start_line..].binary_search(&(span.end - 1)) {
                Ok(end_line) => start_line..end_line + 1,
                Err(insert) => start_line..insert,
            }
        }
    }

    pub fn file_len(&self) -> usize {
        return self.source.len();
    }
}

/// A location within a file,
/// using line-column indexing as opposed to character indices.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Location {
    /// The line of the character
    pub line: usize,
    /// The column of the character on the `line`
    pub column: usize,
}

impl PartialOrd for Location {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(
            match (self.line.cmp(&other.line), self.column.cmp(&other.column)) {
                (Ordering::Equal, Ordering::Equal) => Ordering::Equal,
                (Ordering::Equal, Ordering::Greater) => Ordering::Greater,
                (Ordering::Equal, Ordering::Less) => Ordering::Less,
                (Ordering::Greater, _) => Ordering::Greater,
                (Ordering::Less, _) => Ordering::Less,
            },
        )
    }
}
