//! Token related definitions.
use crate::error::ParseError;

pub mod simple;

/// A token type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
    /// pointer increment (Brainfuck: '>')
    PInc,
    /// pointer decrement (Brainfuck: '<)
    PDec,
    /// data increment (Brainfuck: '+')
    DInc,
    /// data decrement (Brainfuck: '-')
    DDec,
    /// output (Brainfuck: '.')
    Output,
    /// input (Brainfuck: ',')
    Input,
    /// loop head (Brainfuck: '[')
    LoopHead,
    /// loop tail (Brainfuck: ']')
    LoopTail,
}

/// A token information.
#[derive(Debug, PartialEq, Eq)]
pub struct TokenInfo {
    /// The token type. `None` means the EOF.
    pub token_type: Option<TokenType>,
    /// The position of the token in the source string which is counted in Unicode scalar units.
    /// If `token_type` is `None`, this field points to the position of the EOF.
    pub pos_in_chars: usize,
}

/// A tokenizer trait.
///
/// This trait generates a [`TokenStream`] from a source string.
pub trait Tokenizer<'a> {
    type Stream: TokenStream;

    fn token_stream(&'a self, source: &'a str) -> Self::Stream;
}

/// A token stream trait.
///
/// This trait iterates over tokens in the source string.
///
/// # Note
///
/// This is not related with the [`Iterator`] trait.
pub trait TokenStream {
    fn next(&mut self) -> Result<TokenInfo, ParseError>;
}
