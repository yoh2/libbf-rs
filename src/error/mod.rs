//! This module contains error definitions.

use std::io;

use thiserror::Error;

/// A parse error.
///
/// Each variant has the position where the error occurred in Uincode scalar units,
#[derive(Debug, Error)]
pub enum ParseError {
    /// An error returned when a parser unexpectedly reached to "end of file".
    ///
    /// This error typically occurred when a loop was not closed.
    #[error("{pos_in_chars}: Unexpected end-of-file")]
    UnexpectedEndOfFile {
        /// The position where the error occurred.
        pos_in_chars: usize,
    },

    /// An error returned when a parser unexpectedly reached to an end-of-loop.
    ///
    /// This error occurred when end-of-loop token was appeared outside a loop.
    #[error("{pos_in_chars}: Unexpected end-of-loop")]
    UnexpectedEndOfLoop {
        /// The position where the error occurred.
        pos_in_chars: usize,
    },

    /// A miscellaneous error.
    #[error("{pos_in_chars}: syntax error: {message}")]
    MiscError {
        /// The position where the error occurred.
        pos_in_chars: usize,
        /// Error details.
        message: String,
    },
}

/// A parse Error or IO Error.
#[derive(Debug, Error)]
pub enum ParseOrIoError {
    // A parse error.
    #[error("{0}")]
    ParseError(#[from] ParseError),

    // An IO error.
    #[error("{0}")]
    IoError(#[from] io::Error),
}

/// A program runtime error.
#[derive(Debug, Error)]
pub enum RuntimeError {
    /// An error returned when a program accesses a momory that is out of range.
    ///
    /// An "access" occurs when a deta increment/decrement, input or output instruction is performed
    /// and does not occur when the data pointer just points out of range.
    #[error("out of memory bounds [{address}]")]
    OutOfMemoryBounds {
        /// The address where the instruction tried to access.
        address: isize,
    },

    /// An IO error.
    ///
    /// This error occurs when an input or output instruction is failed except end-of-file.
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),

    /// An error returned when an input instruction detects an end-of-file.
    #[error("detected EOF")]
    Eof,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn runtime_error_string() {
        assert_eq!(
            "out of memory bounds [123]",
            RuntimeError::OutOfMemoryBounds { address: 123 }.to_string()
        );
    }
}
