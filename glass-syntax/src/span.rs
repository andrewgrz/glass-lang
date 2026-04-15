//! Location and Span define where a given syntax is located in the filesystem
//!
//! Location is the point in the File System and a Span is from 2 points
//!
//! ```rust
//! use glass_syntax::span::{Span, Location};
//!
//! // The 4th character of the first line in the file
//! let start = Location::new(4, 1, 4);
//! // The 6th character of the second line in the file
//! let end = Location::new(6, 2, 20);
//! let span = Span::new(start, end, "examples/test.gl");
//!
//! println!("{:?}", span);
//! ```

/// The location of a span with the line and offset
/// ```rust
/// use glass_syntax::span::Location;
/// // The 4th character of the first line in the file
/// let start = Location::new(4, 1, 4);
/// ````
#[derive(Debug, Clone, PartialEq)]
pub struct Location {
    /// The number of the line, starting index of 1
    pub line: usize,
    /// The index of the column, starting index of 1
    pub column: usize,
    /// The full character offset from the beginning of the file, starting index of 1
    pub offset: usize,
}

impl Location {
    pub fn new(offset: usize, line: usize, column: usize) -> Location {
        Location {
            offset,
            line,
            column,
        }
    }
}

/// A span for a AST node.
///
/// Must be contained with only within 1 file
///
/// ```rust
/// use glass_syntax::span::{Span, Location};
///
/// // The 4th character of the first line in the file
/// let start = Location::new(4, 1, 4);
/// // The 6th character of the second line in the file
/// let end = Location::new(6, 2, 20);
/// let span = Span::new(start, end, "examples/test.gl");
///
/// println!("{:?}", span);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Span {
    /// The starting location of the span
    pub start: Location,
    /// The ending location of the span
    pub end: Location,
    /// The filename of location of the span
    pub filename: String,
}

impl Span {
    /// Create a new span from the given location
    pub fn new(start: Location, end: Location, filename: &str) -> Span {
        Span {
            start,
            end,
            filename: filename.into(),
        }
    }
}

/// A Factory for easily creating new spans from the last generated Span in the File
pub struct SpanFactory {
    current: Location,
    filename: String,
}

impl SpanFactory {
    pub fn new(filename: &str) -> SpanFactory {
        SpanFactory {
            current: Location { line: 1, column: 1, offset: 0 },
            filename: filename.into(),
        }
    }

    pub fn span(&mut self, num_lines: usize, num_cols: usize) -> Span {
        let start_location = self.current.clone();
        let mut end_location = self.current.clone();
        end_location.line += num_lines;
        end_location.column += num_cols;
        end_location.offset += num_cols;
        self.current = end_location.clone();

        Span {
            start: start_location,
            end: end_location,
            filename: self.filename.clone(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
}
