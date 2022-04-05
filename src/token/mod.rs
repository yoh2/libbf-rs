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
}

/// A token information.
pub struct TokenInfo {
    /// The token type. `None` means the EOF.
    pub token_type: Option<TokenType>,
    /// The position of the token in the source string which is counted in Unicode scalar units.
    /// If `token_type` is `None`, this field points to the position of the EOF.
    pub pos_in_chars: usize,
}

pub trait Tokenizer<'a> {
    type Stream: TokenStream;

    fn token_stream(&'a self, source: &'a str) -> Self::Stream;
}

pub trait TokenStream {
    fn next(&mut self) -> Result<TokenInfo, ParseError>;
}
