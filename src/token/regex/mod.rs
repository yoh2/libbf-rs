//! Regex tokenizers.
//!
//! # Examples
//!
//! ```
//! use libbf::{
//!     token::{
//!         regex::{
//!            RegexTokenizer,
//!         },
//!         Token,
//!         TokenStream,
//!         TokenType,
//!         TokenInfo,
//!         Tokenizer,
//!     },
//! };
//!
//! let tokenizer = RegexTokenizer::from_str_spec(&[
//!     (TokenType::PInc, r#"[>＞]"#),
//!     (TokenType::PDec, r#"[<＜]"#),
//!     (TokenType::DInc, r#"[+＋]"#),
//!     (TokenType::DDec, r#"[-−]"#),
//!     (TokenType::Output, r#"[,，]"#),
//!     (TokenType::Input, r#"[.．]"#),
//!     (TokenType::LoopHead, r#"[\[［]"#),
//!     (TokenType::LoopTail, r#"[］]"#),
//! ]).unwrap();
//!
//! let mut stream = tokenizer.token_stream("＞（ｉｇｎｏｒｅｄ　ｈｅｒｅ）＋");
//!
//! assert_eq!(
//!     stream.next().unwrap(),
//!     TokenInfo {
//!         token: Some(Token {
//!             token_type: TokenType::PInc,
//!             token_str: "＞",
//!         }),
//!         pos_in_chars: 0,
//!     },
//! );
//! assert_eq!(
//!     stream.next().unwrap(),
//!     TokenInfo {
//!         token: Some(Token {
//!             token_type: TokenType::DInc,
//!             token_str: "＋",
//!         }),
//!         pos_in_chars: 15,
//!     },
//! );
//! assert_eq!(
//!     stream.next().unwrap(),
//!     TokenInfo {
//!         token: None,
//!         pos_in_chars: 16,
//!     },
//! );
//! ```

use super::{Token, TokenInfo, TokenStream, TokenType, Tokenizer};
use crate::error::ParseError;
use regex::{Match, Regex};
use thiserror::Error;

/// An error that occurred during compiling regular expressions in a token specification.
#[derive(Debug, Error)]
#[error("Regex errors: {0:?}")]
pub struct RegexErrors(pub Vec<RegexErrorDescription>);

/// An element of [`RegexErrors`] field.
#[derive(Debug)]
pub struct RegexErrorDescription {
    /// The index of a token specification which caused the error.
    pub index: usize,
    /// The caused error.
    pub error: regex::Error,
}

struct RegexTokenDef {
    token_type: TokenType,
    regex: Regex,
}

/// A tokenizer that each token is represented in a regular expression.
pub struct RegexTokenizer {
    token_defs: Vec<RegexTokenDef>,
}

impl RegexTokenizer {
    /// Create a new [`RegexTokenizer`] with pairs of [`TokenType`] and [`Regex`].
    ///
    /// Each regex will be cloned.
    pub fn new(spec: &[(TokenType, Regex)]) -> Self {
        let token_defs = spec
            .iter()
            .map(|(t, r)| RegexTokenDef {
                token_type: *t,
                regex: r.clone(),
            })
            .collect();
        Self { token_defs }
    }

    /// Create a new [`RegexTokenizer`] with pairs of [`TokenType`] and string.
    ///
    /// Passed string is compiled as a regular expression.
    ///
    /// # Errors
    ///
    /// If some compile errors occur, these errors are collected into [`RegexErrors`] and returned.
    pub fn from_str_spec(spec: &[(TokenType, &str)]) -> Result<Self, RegexErrors> {
        let mut token_defs = Vec::with_capacity(spec.len());
        let mut errors = Vec::new();

        for (index, (token_type, re)) in spec.iter().enumerate() {
            match Regex::new(re) {
                Ok(regex) => {
                    token_defs.push(RegexTokenDef {
                        token_type: *token_type,
                        regex,
                    });
                }
                Err(error) => errors.push(RegexErrorDescription { index, error }),
            }
        }

        if !errors.is_empty() {
            Err(RegexErrors(errors))
        } else {
            Ok(Self { token_defs })
        }
    }
}

impl<'a> Tokenizer<'a> for RegexTokenizer {
    type Stream = RegexTokenStream<'a>;

    fn token_stream(&'a self, source: &'a str) -> Self::Stream {
        RegexTokenStream {
            token_defs: &self.token_defs,
            source,
            pos: 0,
            pos_in_chars: 0,
        }
    }
}

/// A token stream generated by [`RegexTokenizer`].
///
/// The next token is determined as follows:
///
/// 1. The token with the smallest matching position is taken as the next token.
/// 2. If multiple tokens have the smallest matching position, the token with
///    the smallest index in the token definitions[^token_def] among these tokens is taken.
/// 3. If no tokens match, it is determined that the end of file has been reached.
///
/// [^token_def]: The order of the token definitions is the same as the order of
/// specifications that was passed when [`RegexTokenizer`] which was used to generate
/// the stream was created.
pub struct RegexTokenStream<'a> {
    token_defs: &'a [RegexTokenDef],
    source: &'a str,
    pos: usize,
    pos_in_chars: usize,
}

impl<'a> TokenStream<'a> for RegexTokenStream<'a> {
    fn next(&mut self) -> Result<TokenInfo<'a>, ParseError> {
        let subtext = &self.source[self.pos..];
        match next_match(subtext, self.token_defs) {
            Some((m, def)) => {
                let matched_str = m.as_str();
                let pos_in_chars = self.pos_in_chars + subtext[..m.start()].chars().count();

                self.pos += m.end();
                self.pos_in_chars = pos_in_chars + matched_str.chars().count();

                Ok(TokenInfo {
                    token: Some(Token {
                        token_type: def.token_type,
                        token_str: m.as_str(),
                    }),
                    pos_in_chars,
                })
            }
            None => {
                self.pos = self.source.len();
                self.pos_in_chars += subtext.chars().count();
                Ok(TokenInfo {
                    token: None,
                    pos_in_chars: self.pos_in_chars,
                })
            }
        }
    }
}

fn next_match<'a, 'b>(
    text: &'a str,
    token_defs: &'b [RegexTokenDef],
) -> Option<(Match<'a>, &'b RegexTokenDef)> {
    token_defs
        .iter()
        .filter_map(|def| def.regex.find(text).map(|m| (m, def)))
        .min_by_key(|&(m, _)| m.start())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_tokenizer_from_str_spec() {
        let bad_tokenizer_result = RegexTokenizer::from_str_spec(&[
            (TokenType::PInc, r#">"#),
            (TokenType::PDec, r#"<"#),
            (TokenType::DInc, r#"+"#), // bad
            (TokenType::DDec, r#"-"#),
            (TokenType::Output, r#","#),
            (TokenType::Input, r#"\."#),
            (TokenType::LoopHead, r#"["#), // bad
            (TokenType::LoopTail, r#"]"#),
        ]);

        if let Err(RegexErrors(es)) = bad_tokenizer_result {
            assert_eq!(es.len(), 2);
            assert_eq!(es[0].index, 2);
            assert_eq!(es[1].index, 6);
        } else {
            panic!("unexpected result");
        }

        let ok_tokenizer_result = RegexTokenizer::from_str_spec(&[
            (TokenType::PInc, r#">"#),
            (TokenType::PDec, r#"<"#),
            (TokenType::DInc, r#"\+"#),
            (TokenType::DDec, r#"-"#),
            (TokenType::Output, r#","#),
            (TokenType::Input, r#"\."#),
            (TokenType::LoopHead, r#"\["#), // bad
            (TokenType::LoopTail, r#"]"#),
        ]);
        assert!(ok_tokenizer_result.is_ok());
    }

    #[test]
    fn test_stream() {
        let tokenizer = RegexTokenizer::from_str_spec(&[
            (TokenType::PInc, r"[>＞]]"),
            (TokenType::PDec, r"[<＜]"),
            (TokenType::DInc, r"[+＋]"),
            (TokenType::DDec, r"[-−]"),
            (TokenType::Input, r"[,，]"),
            (TokenType::Output, r"[.．]"),
            (TokenType::LoopHead, r"[\[［]"),
            (TokenType::LoopTail, r"[]］]"),
        ])
        .expect("all regexes should be compiled successfully");
        // byte 0 (char 0): DInc
        // byte 10 (char 4): DDec
        // byte 22 (char 8): EOF
        let mut stream = tokenizer.token_stream("+ａｂｃ−ｄｅｆ");
        assert_eq!(
            stream.next().unwrap(),
            TokenInfo {
                token: Some(Token {
                    token_type: TokenType::DInc,
                    token_str: "+",
                }),
                pos_in_chars: 0,
            },
        );
        assert_eq!(
            stream.next().unwrap(),
            TokenInfo {
                token: Some(Token {
                    token_type: TokenType::DDec,
                    token_str: "−",
                }),
                pos_in_chars: 4,
            },
        );
        assert_eq!(
            stream.next().unwrap(),
            TokenInfo {
                token: None,
                pos_in_chars: 8,
            },
        );
    }
}
