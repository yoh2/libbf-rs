//! Token related definitions.
#![cfg_attr(docsrs, feature(doc_cfg))]

use crate::error::ParseError;

#[cfg(feature = "regex")]
#[cfg_attr(docsrs, doc(cfg(feature = "regex")))]
pub mod regex;
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

/// A token.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Token<'a> {
    /// The token type.
    pub token_type: TokenType,
    /// The token string.
    pub token_str: &'a str,
}

/// A token information.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct TokenInfo<'a> {
    /// The token. `None` means the EOF.
    pub token: Option<Token<'a>>,
    /// The position of the token in the source string which is counted in Unicode scalar units.
    /// If `token_type` is `None`, this field points to the position of the EOF.
    pub pos_in_chars: usize,
}

impl<'a> TokenInfo<'a> {
    pub fn token_type(&self) -> Option<TokenType> {
        self.token.as_ref().map(|token| token.token_type)
    }

    pub fn token_str(&self) -> Option<&'a str> {
        self.token.as_ref().map(|token| token.token_str)
    }
}

/// A tokenizer trait.
///
/// This trait generates a [`TokenStream`] from a source string.
pub trait Tokenizer<'a> {
    type Stream: TokenStream<'a>;

    fn token_stream(&'a self, source: &'a str) -> Self::Stream;
}

/// A token stream trait.
///
/// This trait iterates over tokens in the source string.
///
/// # Note
///
/// This is not related with the [`Iterator`] trait.
pub trait TokenStream<'a> {
    fn next(&mut self) -> Result<TokenInfo<'a>, ParseError>;
}
