//! Simple tokenizers.
//!
//! # Example
//!
//! ```
//! use libbf::{
//!     token::{
//!         Token,
//!         TokenType,
//!         Tokenizer,
//!         TokenStream,
//!         simple::{
//!             SimpleTokenSpec,
//!         },
//!         TokenInfo,
//!     },
//! };
//!
//! let tokenizer = SimpleTokenSpec {
//!     ptr_inc: '>',
//!     ptr_dec: '<',
//!     data_inc: '+',
//!     data_dec: '-',
//!     output: '.',
//!     input: ',',
//!     loop_head: '[',
//!     loop_tail: ']',
//! }.to_tokenizer();
//!
//! let mut stream = tokenizer.token_stream(">(ignored here)+");
//!
//! assert_eq!(
//!     stream.next().unwrap(),
//!     TokenInfo {
//!         token: Some(Token {
//!             token_type: TokenType::PInc,
//!             token_str: ">",
//!         }),
//!         pos_in_chars: 0,
//!     },
//! );
//! assert_eq!(
//!     stream.next().unwrap(),
//!     TokenInfo {
//!         token: Some(Token {
//!             token_type: TokenType::DInc,
//!             token_str: "+",
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

/// A token specification for [`SimpleTokenizer`].
pub struct SimpleTokenSpec<S1, S2, S3, S4, S5, S6, S7, S8> {
    /// The token representing pointer increment (`>').
    pub ptr_inc: S1,
    /// The token representing pointer decrement (`<').
    pub ptr_dec: S2,
    /// The token representing data increment (`+').
    pub data_inc: S3,
    /// The token representing data decrement (`-').
    pub data_dec: S4,
    /// The token representing output (`.`).
    pub output: S5,
    /// The token representing input (`,`).
    pub input: S6,
    /// The token representing loop head (`[`).
    pub loop_head: S7,
    /// The token representing loop tail (`]`).
    pub loop_tail: S8,
}

/// A variant of [`SimpleTokenSpec`] where all members have the same type.
pub type SimpleTokenSpec1<S> = SimpleTokenSpec<S, S, S, S, S, S, S, S>;

impl<S1, S2, S3, S4, S5, S6, S7, S8> SimpleTokenSpec<S1, S2, S3, S4, S5, S6, S7, S8>
where
    S1: ToString,
    S2: ToString,
    S3: ToString,
    S4: ToString,
    S5: ToString,
    S6: ToString,
    S7: ToString,
    S8: ToString,
{
    pub fn to_tokenizer(&self) -> SimpleTokenizer {
        let mut token_table = vec![
            SimpleTokenDef::new(&self.ptr_inc, TokenType::PInc),
            SimpleTokenDef::new(&self.ptr_dec, TokenType::PDec),
            SimpleTokenDef::new(&self.data_inc, TokenType::DInc),
            SimpleTokenDef::new(&self.data_dec, TokenType::DDec),
            SimpleTokenDef::new(&self.output, TokenType::Output),
            SimpleTokenDef::new(&self.input, TokenType::Input),
            SimpleTokenDef::new(&self.loop_head, TokenType::LoopHead),
            SimpleTokenDef::new(&self.loop_tail, TokenType::LoopTail),
        ];
        // Sort the table by token length in descending order in order to fetch token by longest match strategy.
        token_table.sort_by_key(|def| usize::MAX - def.char_count);
        SimpleTokenizer { token_table }
    }
}

#[test]
fn test_simple_def_to_tokenizer() {
    let spec = SimpleTokenSpec {
        ptr_inc: "??????",
        ptr_dec: "aaaaa",
        data_inc: '???',
        data_dec: "?????????",
        output: "????????????",
        input: "dddddddd".to_string(),
        loop_head: "ccccccc",
        loop_tail: "bbbbbb",
    };
    let tokenizer = spec.to_tokenizer();
    let expected = [
        SimpleTokenDef {
            token: "dddddddd".to_string(),
            token_type: TokenType::Input,
            char_count: 8,
        },
        SimpleTokenDef {
            token: "ccccccc".to_string(),
            token_type: TokenType::LoopHead,
            char_count: 7,
        },
        SimpleTokenDef {
            token: "bbbbbb".to_string(),
            token_type: TokenType::LoopTail,
            char_count: 6,
        },
        SimpleTokenDef {
            token: "aaaaa".to_string(),
            token_type: TokenType::PDec,
            char_count: 5,
        },
        SimpleTokenDef {
            token: "????????????".to_string(),
            token_type: TokenType::Output,
            char_count: 4,
        },
        SimpleTokenDef {
            token: "?????????".to_string(),
            token_type: TokenType::DDec,
            char_count: 3,
        },
        SimpleTokenDef {
            token: "??????".to_string(),
            token_type: TokenType::PInc,
            char_count: 2,
        },
        SimpleTokenDef {
            token: "???".to_string(),
            token_type: TokenType::DInc,
            char_count: 1,
        },
    ];
    assert_simple_def_eq(&tokenizer.token_table, &expected);
}

#[cfg(test)]
fn assert_simple_def_eq(actual: &[SimpleTokenDef], expected: &[SimpleTokenDef]) {
    assert_eq!(actual.len(), expected.len(), "length");
    for (index, (a, e)) in actual.iter().zip(expected.iter()).enumerate() {
        assert_eq!(a.token, e.token, "[{index}].token");
        assert_eq!(a.token_type, e.token_type, "[{index}].token_type");
        assert_eq!(a.char_count, e.char_count, "[{index}].char_count");
    }
}

/// Token specification for [`SimpleTokenizer`] which allows have multiple tokens for the same token type.
pub struct SimpleMultiTokenSpec<'a, S1, S2, S3, S4, S5, S6, S7, S8> {
    /// Tokens representing pointer increment (`>').
    pub ptr_inc: &'a [S1],
    /// Tokens representing pointer decrement (`<').
    pub ptr_dec: &'a [S2],
    /// Tokens representing data increment (`+').
    pub data_inc: &'a [S3],
    /// Tokens representing data decrement (`-').
    pub data_dec: &'a [S4],
    /// Tokens representing output (`.`).
    pub output: &'a [S5],
    /// Tokens representing input (`,`).
    pub input: &'a [S6],
    /// Tokens representing loop head (`[`).
    pub loop_head: &'a [S7],
    /// Tokens representing loop tail (`]`).
    pub loop_tail: &'a [S8],
}

/// A variant of [`SimpleMultiTokenSpec`] where all members have the same type.
pub type SimpleMultiTokenSpec1<'a, S> = SimpleMultiTokenSpec<'a, S, S, S, S, S, S, S, S>;

impl<'a, S1, S2, S3, S4, S5, S6, S7, S8> SimpleMultiTokenSpec<'a, S1, S2, S3, S4, S5, S6, S7, S8>
where
    S1: ToString,
    S2: ToString,
    S3: ToString,
    S4: ToString,
    S5: ToString,
    S6: ToString,
    S7: ToString,
    S8: ToString,
{
    pub fn to_tokenizer(&self) -> SimpleTokenizer {
        let mut token_table = Self::to_token_defs(self.ptr_inc, TokenType::PInc)
            .chain(Self::to_token_defs(self.ptr_dec, TokenType::PDec))
            .chain(Self::to_token_defs(self.data_inc, TokenType::DInc))
            .chain(Self::to_token_defs(self.data_dec, TokenType::DDec))
            .chain(Self::to_token_defs(self.output, TokenType::Output))
            .chain(Self::to_token_defs(self.input, TokenType::Input))
            .chain(Self::to_token_defs(self.loop_head, TokenType::LoopHead))
            .chain(Self::to_token_defs(self.loop_tail, TokenType::LoopTail))
            .collect::<Vec<_>>();
        // Sort the table by token length in descending order in order to fetch token by longest match strategy.
        token_table.sort_by_key(|def| usize::MAX - def.char_count);
        SimpleTokenizer { token_table }
    }

    fn to_token_defs(
        tokens: &[impl ToString],
        token_type: TokenType,
    ) -> impl Iterator<Item = SimpleTokenDef> + '_ {
        tokens
            .iter()
            .map(move |token| SimpleTokenDef::new(token, token_type))
    }
}

#[test]
fn test_multiple_simple_def_to_tokenizer() {
    let spec = SimpleMultiTokenSpec {
        ptr_inc: &["??????"],
        ptr_dec: &["aaaaa"],
        data_inc: &['???'],
        data_dec: &["?????????", "??????????"],
        output: &["????????????"],
        input: &["dddddddd".to_string()],
        loop_head: &["ccccccc"],
        loop_tail: &["bbbbbb"],
    };
    let tokenizer = spec.to_tokenizer();
    let expected = [
        SimpleTokenDef {
            token: "??????????".to_string(),
            token_type: TokenType::DDec,
            char_count: 10,
        },
        SimpleTokenDef {
            token: "dddddddd".to_string(),
            token_type: TokenType::Input,
            char_count: 8,
        },
        SimpleTokenDef {
            token: "ccccccc".to_string(),
            token_type: TokenType::LoopHead,
            char_count: 7,
        },
        SimpleTokenDef {
            token: "bbbbbb".to_string(),
            token_type: TokenType::LoopTail,
            char_count: 6,
        },
        SimpleTokenDef {
            token: "aaaaa".to_string(),
            token_type: TokenType::PDec,
            char_count: 5,
        },
        SimpleTokenDef {
            token: "????????????".to_string(),
            token_type: TokenType::Output,
            char_count: 4,
        },
        SimpleTokenDef {
            token: "?????????".to_string(),
            token_type: TokenType::DDec,
            char_count: 3,
        },
        SimpleTokenDef {
            token: "??????".to_string(),
            token_type: TokenType::PInc,
            char_count: 2,
        },
        SimpleTokenDef {
            token: "???".to_string(),
            token_type: TokenType::DInc,
            char_count: 1,
        },
    ];
    assert_simple_def_eq(&tokenizer.token_table, &expected);
}

// Token definition
#[derive(Debug, PartialEq, Eq)]
struct SimpleTokenDef {
    // The token string
    token: String,
    // The precomputed token.chars().count()
    char_count: usize,
    // The token type
    token_type: TokenType,
}

impl SimpleTokenDef {
    fn new(token: &impl ToString, token_type: TokenType) -> Self {
        let token = token.to_string();
        let char_count = token.chars().count();
        Self {
            token,
            char_count,
            token_type,
        }
    }
}

/// A tokenizer that each token in source code simply corresponds to a single [`TokenType`].
pub struct SimpleTokenizer {
    token_table: Vec<SimpleTokenDef>,
}

impl<'a> Tokenizer<'a> for SimpleTokenizer {
    type Stream = SimpleTokenStream<'a>;

    fn token_stream(&'a self, source: &'a str) -> SimpleTokenStream<'a> {
        SimpleTokenStream::new(source, &self.token_table)
    }
}

/// A token stream generated by [`SimpleTokenizer`].
pub struct SimpleTokenStream<'a> {
    token_table: &'a [SimpleTokenDef],
    source: &'a str,
    pos: usize,
    pos_in_chars: usize,
}

impl<'a> SimpleTokenStream<'a> {
    fn new(source: &'a str, token_table: &'a [SimpleTokenDef]) -> Self {
        SimpleTokenStream {
            token_table,
            source,
            pos: 0,
            pos_in_chars: 0,
        }
    }
}

impl<'a> TokenStream<'a> for SimpleTokenStream<'a> {
    fn next(&mut self) -> Result<TokenInfo<'a>, crate::error::ParseError> {
        // TODO: This loop is too dumb. It should use more efficient algorithm.

        let mut rel_pos_in_chars = 0;
        for (rel_pos, _) in self.source[self.pos..].char_indices() {
            let pos = self.pos + rel_pos;
            if let Some(def) = find_token_at(self.source, pos, self.token_table) {
                let info = TokenInfo {
                    token: Some(Token {
                        token_type: def.token_type,
                        token_str: &self.source[pos..pos + def.token.len()],
                    }),
                    pos_in_chars: self.pos_in_chars + rel_pos_in_chars,
                };
                // next position
                self.pos = pos + def.token.len();
                self.pos_in_chars += rel_pos_in_chars + def.char_count;
                return Ok(info);
            }
            rel_pos_in_chars += 1;
        }

        // Token not found.
        // Set the current position to EOF.
        self.pos = self.source.len();
        self.pos_in_chars += rel_pos_in_chars;

        Ok(TokenInfo {
            token: None,
            pos_in_chars: self.pos_in_chars,
        })
    }
}

fn find_token_at<'a>(
    source: &str,
    pos: usize,
    token_table: &'a [SimpleTokenDef],
) -> Option<&'a SimpleTokenDef> {
    let src_head = &source[pos..];
    token_table
        .iter()
        .find(|def| src_head.starts_with(&def.token))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_token_stream() {
        // each token is fullwidth (multi-byte) character.
        let spec = SimpleTokenSpec {
            ptr_inc: '???',
            ptr_dec: '???',
            data_inc: '???',
            data_dec: '???',
            output: '???',
            input: '???',
            loop_head: '???',
            loop_tail: '???',
        };
        let tokenizer = spec.to_tokenizer();
        // byte 0 (char 0): PDec
        // byte 14 (char 6): PInc
        // byte 26 (char 10): EOF
        let mut stream = tokenizer.token_stream("???-?????????+????????????");
        assert_eq!(
            stream.next().unwrap(),
            TokenInfo {
                token: Some(Token {
                    token_type: TokenType::PDec,
                    token_str: "???",
                }),
                pos_in_chars: 0,
            }
        );
        assert_eq!(
            stream.next().unwrap(),
            TokenInfo {
                token: Some(Token {
                    token_type: TokenType::PInc,
                    token_str: "???",
                }),
                pos_in_chars: 6,
            }
        );
        assert_eq!(
            stream.next().unwrap(),
            TokenInfo {
                token: None,
                pos_in_chars: 10,
            }
        );
    }
}
