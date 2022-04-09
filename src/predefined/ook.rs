//! Predefined Ook! implementations.
use crate::{
    error::ParseError,
    prelude::Parser,
    token::{TokenInfo, TokenStream, TokenType, Tokenizer},
};

#[derive(Debug, Clone, Copy)]
enum OokTokenType {
    /// Ook.
    Dot,
    /// Ook?
    Question,
    /// Ook!
    Exclamation,
}

struct OokTokenInfo {
    token_type: Option<OokTokenType>,
    /// The position of the token in the source.
    pos_in_chars: usize,
}

/// Create a parser for Ook!
pub fn parser() -> Parser<OokTokenizer> {
    Parser::new(OokTokenizer)
}

/// A tokenizer for Ook!
pub struct OokTokenizer;

impl<'a> Tokenizer<'a> for OokTokenizer {
    type Stream = OokTokenStream<'a>;

    fn token_stream(&'a self, source: &'a str) -> Self::Stream {
        OokTokenStream::new(source)
    }
}

/// A token stream for Ook!
pub struct OokTokenStream<'a> {
    source: &'a str,
    pos: usize,
    pos_in_chars: usize,
}

const COMMON_TOKEN_PART: &str = "Ook";

impl<'a> OokTokenStream<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            source,
            pos: 0,
            pos_in_chars: 0,
        }
    }

    fn next_ook_token(&mut self) -> OokTokenInfo {
        let mut rel_pos_in_chars = 0;
        for (rel_pos, _) in self.source[self.pos..].char_indices() {
            let src_head = &self.source[self.pos + rel_pos..];
            if let Some(s) = src_head.strip_prefix(COMMON_TOKEN_PART) {
                let token_type = match s.chars().next() {
                    Some('.') => OokTokenType::Dot,
                    Some('?') => OokTokenType::Question,
                    Some('!') => OokTokenType::Exclamation,
                    _ => {
                        rel_pos_in_chars += 1;
                        continue;
                    }
                };
                let info = OokTokenInfo {
                    token_type: Some(token_type),
                    pos_in_chars: self.pos_in_chars + rel_pos_in_chars,
                };
                // next position
                self.pos += rel_pos + COMMON_TOKEN_PART.len() + 1;
                self.pos_in_chars += rel_pos_in_chars + COMMON_TOKEN_PART.len() + 1;
                return info;
            }
            rel_pos_in_chars += 1;
        }

        // Token not found.
        // Set the current position to EOF.
        self.pos = self.source.len();
        self.pos_in_chars += rel_pos_in_chars;

        OokTokenInfo {
            token_type: None,
            pos_in_chars: self.pos_in_chars,
        }
    }
}

impl<'a> TokenStream for OokTokenStream<'a> {
    fn next(&mut self) -> Result<TokenInfo, ParseError> {
        let (first_token_type, first_token_pos_in_chars) = {
            let token = self.next_ook_token();
            if let Some(token_type) = token.token_type {
                (token_type, token.pos_in_chars)
            } else {
                return Ok(TokenInfo {
                    token_type: None,
                    pos_in_chars: token.pos_in_chars,
                });
            }
        };

        let second_token_type = {
            let token = self.next_ook_token();
            if let Some(token_type) = token.token_type {
                token_type
            } else {
                return Err(ParseError::MiscError(
                    token.pos_in_chars,
                    "Odd number of Ook tokens".to_string(),
                ));
            }
        };

        let token_type = match (first_token_type, second_token_type) {
            (OokTokenType::Dot, OokTokenType::Question) => TokenType::PInc,
            (OokTokenType::Question, OokTokenType::Dot) => TokenType::PDec,
            (OokTokenType::Dot, OokTokenType::Dot) => TokenType::DInc,
            (OokTokenType::Exclamation, OokTokenType::Exclamation) => TokenType::DDec,
            (OokTokenType::Exclamation, OokTokenType::Dot) => TokenType::Output,
            (OokTokenType::Dot, OokTokenType::Exclamation) => TokenType::Input,
            (OokTokenType::Exclamation, OokTokenType::Question) => TokenType::LoopHead,
            (OokTokenType::Question, OokTokenType::Exclamation) => TokenType::LoopTail,
            (OokTokenType::Question, OokTokenType::Question) => {
                return Err(ParseError::MiscError(
                    first_token_pos_in_chars,
                    "Ook? Ook?: bad Ook sequence".to_string(),
                ))
            }
        };

        Ok(TokenInfo {
            token_type: Some(token_type),
            pos_in_chars: first_token_pos_in_chars,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::runtime;

    #[test]
    fn test_hello_world() {
        // source code from https://esolangs.org/wiki/Ook!
        let source = r##"
            Ook. Ook? Ook. Ook. Ook. Ook. Ook. Ook. Ook. Ook. Ook. Ook. Ook. Ook. Ook. Ook.
            Ook. Ook. Ook. Ook. Ook! Ook? Ook? Ook. Ook. Ook. Ook. Ook. Ook. Ook. Ook. Ook.
            Ook. Ook. Ook. Ook. Ook. Ook. Ook. Ook. Ook. Ook? Ook! Ook! Ook? Ook! Ook? Ook.
            Ook! Ook. Ook. Ook? Ook. Ook. Ook. Ook. Ook. Ook. Ook. Ook. Ook. Ook. Ook. Ook.
            Ook. Ook. Ook! Ook? Ook? Ook. Ook. Ook. Ook. Ook. Ook. Ook. Ook. Ook. Ook. Ook?
            Ook! Ook! Ook? Ook! Ook? Ook. Ook. Ook. Ook! Ook. Ook. Ook. Ook. Ook. Ook. Ook.
            Ook. Ook. Ook. Ook. Ook. Ook. Ook. Ook. Ook! Ook. Ook! Ook. Ook. Ook. Ook. Ook.
            Ook. Ook. Ook! Ook. Ook. Ook? Ook. Ook? Ook. Ook? Ook. Ook. Ook. Ook. Ook. Ook.
            Ook. Ook. Ook. Ook. Ook. Ook. Ook. Ook. Ook. Ook. Ook! Ook? Ook? Ook. Ook. Ook.
            Ook. Ook. Ook. Ook. Ook. Ook. Ook. Ook? Ook! Ook! Ook? Ook! Ook? Ook. Ook! Ook.
            Ook. Ook? Ook. Ook? Ook. Ook? Ook. Ook. Ook. Ook. Ook. Ook. Ook. Ook. Ook. Ook.
            Ook. Ook. Ook. Ook. Ook. Ook. Ook. Ook. Ook. Ook. Ook! Ook? Ook? Ook. Ook. Ook.
            Ook. Ook. Ook. Ook. Ook. Ook. Ook. Ook. Ook. Ook. Ook. Ook. Ook. Ook. Ook. Ook.
            Ook. Ook? Ook! Ook! Ook? Ook! Ook? Ook. Ook! Ook! Ook! Ook! Ook! Ook! Ook! Ook.
            Ook? Ook. Ook? Ook. Ook? Ook. Ook? Ook. Ook! Ook. Ook. Ook. Ook. Ook. Ook. Ook.
            Ook! Ook. Ook! Ook! Ook! Ook! Ook! Ook! Ook! Ook! Ook! Ook! Ook! Ook! Ook! Ook.
            Ook! Ook! Ook! Ook! Ook! Ook! Ook! Ook! Ook! Ook! Ook! Ook! Ook! Ook! Ook! Ook!
            Ook! Ook. Ook. Ook? Ook. Ook? Ook. Ook. Ook! Ook.
        "##;
        let program = match parser().parse_str(source) {
            Ok(program) => program,
            Err(err) => panic!("unexpected error: {err}"),
        };

        let input: &[u8] = &[];
        let mut output = vec![];
        if let Err(err) = runtime::run(&program, input, &mut output) {
            panic!("unexpected error: {err}");
        }
        assert_eq!(output, b"Hello World!");
    }

    #[test]
    fn test_odd_ooks() {
        let source = "Ook. Ook? Ook!";
        if let Err(err) = parser().parse_str(source) {
            if let ParseError::MiscError(pos, msg) = err {
                assert_eq!(pos, source.len());
                assert_eq!(msg, "Odd number of Ook tokens");
            } else {
                panic!("unexpected error: {err}");
            }
        } else {
            assert!(false, "unexpectedly succeeded");
        }
    }

    #[test]
    fn test_bad_ook_sequence() {
        let source = "Ook. Ook? Ook? Ook?";
        if let Err(err) = parser().parse_str(source) {
            if let ParseError::MiscError(pos, msg) = err {
                assert_eq!(pos, 10);
                assert_eq!(msg, "Ook? Ook?: bad Ook sequence");
            } else {
                panic!("unexpected error: {err}");
            }
        } else {
            assert!(false, "unexpectedly succeeded");
        }
    }
}
