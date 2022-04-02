use crate::error::ParseError;

pub mod simple;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
    /// pointer increment ('>')
    PInc,
    /// pointer decrement ('<)
    PDec,
    /// data increment ('+')
    DInc,
    /// data decrement ('-')
    DDec,
    /// output ('.')
    Output,
    /// input (',')
    Input,
    /// loop head ('[')
    LoopHead,
    /// loop tail (']')
    LoopTail,
    /// end of file
    Eof,
}

/// A token information.
pub struct TokenInfo {
    /// The token type.
    pub token_type: TokenType,
    /// The position of the token in the source string.
    pub pos_in_chars: usize,
}

pub trait Tokenizer<'a> {
    type Stream: TokenStream;

    fn token_stream(&'a self, source: &'a str) -> Self::Stream;
}

pub trait TokenStream {
    fn next(&mut self) -> Result<TokenInfo, ParseError>;
}
