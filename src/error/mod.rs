use std::io;

use thiserror::Error;

/// Parse error
#[derive(Debug, Error)]
pub enum ParseError {
    #[error("{0}: Unexpected end-of-file")]
    UnexpectedEndOfFile(usize),

    #[error("{0}: Unexpected end-of-loop")]
    UnexpectedEndOfLoop(usize),

    #[error("{0}: syntax error: {1}")]
    MiscError(usize, String),
}

#[derive(Debug, Error)]
pub enum ParseOrIoError {
    #[error("{0}")]
    ParseError(#[from] ParseError),

    #[error("{0}")]
    IoError(#[from] io::Error),
}

/// Runtime error
#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("out of memory bounds [{0}]")]
    OutOfMemoryBounds(isize),

    #[error("IO error: {0}")]
    IoError(#[from] io::Error),

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
            RuntimeError::OutOfMemoryBounds(123).to_string()
        );
    }
}
