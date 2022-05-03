//! This module provides a parser for the program.
//!
use std::io::Read;

use crate::{
    error::{ParseError, ParseOrIoError},
    program::{FatInstruction, FatInstructionKind, FatProgram, Instruction, Program},
    token::{TokenInfo, TokenStream, TokenType, Tokenizer},
};

// A context for parsing.
//
// This struct holds a token stream and token unget buffer.
struct ParseContext<'a, T> {
    token_stream: T,

    // (length, char count, token type) of ungot token
    unget_buf: Option<TokenInfo<'a>>,
}

impl<'a, T> ParseContext<'a, T>
where
    T: TokenStream<'a>,
{
    fn new(token_stream: T) -> Self {
        Self {
            token_stream,
            unget_buf: None,
        }
    }

    fn next_token_info(&mut self) -> Result<TokenInfo<'a>, ParseError> {
        if let Some(def) = self.unget_buf.take() {
            return Ok(def);
        }
        self.token_stream.next()
    }

    fn unget_token_info(&mut self, info: TokenInfo<'a>) {
        assert!(self.unget_buf.is_none());
        self.unget_buf = Some(info);
    }
}

/// A parser for the program.
///
/// `Parser` parses program tokens which are provided by [`Tokenizer`] and generates [`Program`]
///
/// # Examples
///
/// ```
/// use libbf::{parser::Parser, program::Instruction::*, token::simple::SimpleTokenSpec};
///
/// // Define basic Brainfuck tokenizer.
/// // (note: feature flag `bf` or `predefined` provides `predefined::bf::tokenizer()`
/// //  which can be used to obtain Brainfuck tokenizer instead of manually specifying like below)
/// let tokenizer = SimpleTokenSpec {
///     ptr_inc: '>',
///     ptr_dec: '<',
///     data_inc: '+',
///     data_dec: '-',
///     output: '.',
///     input: ',',
///     loop_head: '[',
///     loop_tail: ']',
/// }.to_tokenizer();
/// // Create a parser with Brainfuck tokenizer.
/// let parser = Parser::new(tokenizer);
/// // Parse Brainfuck program.
/// let program = parser.parse_str(",[.,]").unwrap();
///
/// assert_eq!(program.instructions(), [Input, UntilZero(vec![Output, Input])]);
/// ```
pub struct Parser<T> {
    tokenizer: T,
}

impl<T> Parser<T>
where
    for<'x> T: Tokenizer<'x>,
{
    /// Creates a new parser.
    ///
    /// # Arguments
    ///
    ///  - `tokenizer`: A tokenizer which provides tokens.
    pub fn new(tokenizer: T) -> Self {
        Self { tokenizer }
    }

    /// Parses a program from a [`Read`] object.
    ///
    /// # Errors
    ///
    ///  - [`ParseOrIoError::IoError'](enum.ParseOrIoError.html#variant.IoError) aaa
    ///  - [`ParseOrIoError'] bbb
    pub fn parse(&self, mut reader: impl Read) -> Result<Program, ParseOrIoError> {
        let mut source = String::new();
        let _ = reader.read_to_string(&mut source)?;
        let program = self.parse_str(&source)?;
        Ok(program)
    }

    /// Parses a program from a string.
    ///
    /// # Arguments
    ///
    ///  - `source`: A program source string.
    ///
    /// # Returns
    ///
    /// A program or a parse error.
    pub fn parse_str<'a>(&'a self, source: &'a str) -> Result<Program, ParseError> {
        let mut context = ParseContext::new(self.tokenizer.token_stream(source));
        Ok(Program::new(Self::parse_internal(&mut context, true)?))
    }

    fn parse_internal<'a>(
        context: &mut ParseContext<'a, impl TokenStream<'a>>,
        top_level: bool,
    ) -> Result<Vec<Instruction>, ParseError> {
        let mut instructions = Vec::new();

        loop {
            let info = context.next_token_info()?;
            let token_type = info.token_type();
            match token_type {
                Some(TokenType::PInc) => Self::push_padd(context, &mut instructions, 1)?,
                Some(TokenType::PDec) => Self::push_padd(context, &mut instructions, -1)?,
                Some(TokenType::DInc) => Self::push_dadd(context, &mut instructions, 1)?,
                Some(TokenType::DDec) => Self::push_dadd(context, &mut instructions, -1)?,
                Some(TokenType::Output) => instructions.push(Instruction::Output),
                Some(TokenType::Input) => instructions.push(Instruction::Input),
                Some(TokenType::LoopHead) => instructions.push(Instruction::UntilZero(
                    Self::parse_internal(context, false)?,
                )),
                Some(TokenType::LoopTail) => {
                    if top_level {
                        return Err(ParseError::UnexpectedEndOfLoop {
                            pos_in_chars: info.pos_in_chars,
                        });
                    } else {
                        return Ok(instructions);
                    }
                }

                None => {
                    return if top_level {
                        Ok(instructions)
                    } else {
                        Err(ParseError::UnexpectedEndOfFile {
                            pos_in_chars: info.pos_in_chars,
                        })
                    }
                }
            }
        }
    }

    fn push_padd<'a>(
        context: &mut ParseContext<'a, impl TokenStream<'a>>,
        instructions: &mut Vec<Instruction>,
        initial_operand: isize,
    ) -> Result<(), ParseError> {
        Self::push_xadd(
            context,
            instructions,
            initial_operand,
            TokenType::PInc,
            TokenType::PDec,
            Instruction::PAdd,
        )
    }

    fn push_dadd<'a>(
        context: &mut ParseContext<'a, impl TokenStream<'a>>,
        instructions: &mut Vec<Instruction>,
        initial_operand: isize,
    ) -> Result<(), ParseError> {
        Self::push_xadd(
            context,
            instructions,
            initial_operand,
            TokenType::DInc,
            TokenType::DDec,
            Instruction::DAdd,
        )
    }

    fn push_xadd<'a>(
        context: &mut ParseContext<'a, impl TokenStream<'a>>,
        instructions: &mut Vec<Instruction>,
        initial_operand: isize,
        inc: TokenType,
        dec: TokenType,
        gen: fn(isize) -> Instruction,
    ) -> Result<(), ParseError> {
        let mut operand = initial_operand;

        loop {
            let info = context.next_token_info()?;
            let token_type = info.token_type();
            if token_type == Some(inc) {
                operand += 1;
            } else if token_type == Some(dec) {
                operand -= 1;
            } else {
                // unget token other than inc or dec (including EOF.)
                context.unget_token_info(info);
                break;
            }
        }

        if operand != 0 {
            instructions.push(gen(operand));
        }
        Ok(())
    }

    /// Parses a program from a string.
    ///
    /// # Arguments
    ///
    ///  - `source`: A fat program source string.
    ///
    /// # Returns
    ///
    /// A program or a parse error.
    pub fn parse_str_fat<'a>(&'a self, source: &'a str) -> Result<FatProgram, ParseError> {
        let mut context = ParseContext::new(self.tokenizer.token_stream(source));
        let ParsedFatValue {
            instructions,
            last_token,
        } = Self::parse_internal_fat(&mut context, true)?;
        assert!(last_token.is_none());
        Ok(FatProgram::new(instructions))
    }

    fn parse_internal_fat<'a>(
        context: &mut ParseContext<'a, impl TokenStream<'a>>,
        top_level: bool,
    ) -> Result<ParsedFatValue<'a>, ParseError> {
        let mut instructions = Vec::new();

        loop {
            let info = context.next_token_info()?;
            let token_type = info.token_type();
            match token_type {
                Some(TokenType::PInc) => Self::push_padd_fat(context, &mut instructions, info, 1)?,
                Some(TokenType::PDec) => Self::push_padd_fat(context, &mut instructions, info, -1)?,
                Some(TokenType::DInc) => Self::push_dadd_fat(context, &mut instructions, info, 1)?,
                Some(TokenType::DDec) => Self::push_dadd_fat(context, &mut instructions, info, -1)?,
                Some(TokenType::Output) => instructions.push(FatInstruction {
                    kind: FatInstructionKind::Output,
                    tokens: vec![info],
                }),
                Some(TokenType::Input) => instructions.push(FatInstruction {
                    kind: FatInstructionKind::Input,
                    tokens: vec![info],
                }),
                Some(TokenType::LoopHead) => {
                    let ParsedFatValue {
                        instructions: sub,
                        last_token,
                    } = Self::parse_internal_fat(context, false)?;
                    let mut tokens = Vec::with_capacity(sub.len() + 2);
                    tokens.push(info);
                    tokens.extend(sub.iter().flat_map(|i| i.tokens.iter()));
                    if let Some(last) = last_token {
                        tokens.push(last);
                    } else {
                        unreachable!("last token must be present");
                    }

                    instructions.push(FatInstruction {
                        kind: FatInstructionKind::UntilZero(sub),
                        tokens,
                    });
                }
                Some(TokenType::LoopTail) => {
                    if top_level {
                        return Err(ParseError::UnexpectedEndOfLoop {
                            pos_in_chars: info.pos_in_chars,
                        });
                    } else {
                        return Ok(ParsedFatValue {
                            instructions,
                            last_token: Some(info),
                        });
                    }
                }

                None => {
                    return if top_level {
                        Ok(ParsedFatValue {
                            instructions,
                            last_token: None,
                        })
                    } else {
                        Err(ParseError::UnexpectedEndOfFile {
                            pos_in_chars: info.pos_in_chars,
                        })
                    }
                }
            }
        }
    }

    fn push_padd_fat<'a>(
        context: &mut ParseContext<'a, impl TokenStream<'a>>,
        instructions: &mut Vec<FatInstruction<'a>>,
        initial_token: TokenInfo<'a>,
        initial_operand: isize,
    ) -> Result<(), ParseError> {
        Self::push_xadd_fat(
            context,
            instructions,
            initial_token,
            initial_operand,
            TokenType::PInc,
            TokenType::PDec,
            FatInstructionKind::PAdd,
        )
    }

    fn push_dadd_fat<'a>(
        context: &mut ParseContext<'a, impl TokenStream<'a>>,
        instructions: &mut Vec<FatInstruction<'a>>,
        initial_token: TokenInfo<'a>,
        initial_operand: isize,
    ) -> Result<(), ParseError> {
        Self::push_xadd_fat(
            context,
            instructions,
            initial_token,
            initial_operand,
            TokenType::DInc,
            TokenType::DDec,
            FatInstructionKind::DAdd,
        )
    }

    fn push_xadd_fat<'a>(
        context: &mut ParseContext<'a, impl TokenStream<'a>>,
        instructions: &mut Vec<FatInstruction<'a>>,
        initial_token: TokenInfo<'a>,
        initial_operand: isize,
        inc: TokenType,
        dec: TokenType,
        gen: fn(isize) -> FatInstructionKind<'a>,
    ) -> Result<(), ParseError> {
        let mut tokens = vec![initial_token];
        let mut operand = initial_operand;

        loop {
            let info = context.next_token_info()?;
            let token_type = info.token_type();
            if token_type == Some(inc) {
                operand += 1;
                tokens.push(info);
            } else if token_type == Some(dec) {
                operand -= 1;
                tokens.push(info);
            } else {
                // unget token other than inc or dec (including EOF.)
                context.unget_token_info(info);
                break;
            }
        }

        let kind = if operand == 0 {
            FatInstructionKind::Nop
        } else {
            gen(operand)
        };
        instructions.push(FatInstruction { kind, tokens });

        Ok(())
    }
}

struct ParsedFatValue<'a> {
    instructions: Vec<FatInstruction<'a>>,
    last_token: Option<TokenInfo<'a>>,
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::token::{Token, TokenType};

    #[derive(Clone, Copy)]
    struct TestTokenizer {
        tokens: &'static [TokenInfo<'static>],
        last_pos_in_chars: usize,
    }

    impl<'a> Tokenizer<'a> for TestTokenizer {
        type Stream = TestStream<std::iter::Cloned<std::slice::Iter<'a, TokenInfo<'a>>>>;

        fn token_stream(&'a self, _source: &'a str) -> Self::Stream {
            TestStream {
                tokens: self.tokens.iter().cloned(),
                last_pos_in_chars: self.last_pos_in_chars,
            }
        }
    }

    struct TestStream<I> {
        tokens: I,
        last_pos_in_chars: usize,
    }

    impl<'a, I> TokenStream<'a> for TestStream<I>
    where
        I: Iterator<Item = TokenInfo<'a>>,
    {
        fn next(&mut self) -> Result<TokenInfo<'a>, ParseError> {
            Ok(self.tokens.next().unwrap_or(TokenInfo {
                token: None,
                pos_in_chars: self.last_pos_in_chars,
            }))
        }
    }

    const fn some_token(token_type: TokenType, token_str: &str) -> Option<Token> {
        Some(Token {
            token_type,
            token_str,
        })
    }

    const TEST_TOKENIZER: TestTokenizer = TestTokenizer {
        tokens: &[
            // PAdd(-1)
            TokenInfo {
                token: some_token(TokenType::PInc, ">"),
                pos_in_chars: 0,
            },
            TokenInfo {
                token: some_token(TokenType::PDec, "<"),
                pos_in_chars: 1,
            },
            TokenInfo {
                token: some_token(TokenType::PDec, "<"),
                pos_in_chars: 2,
            },
            // DAdd(1)
            TokenInfo {
                token: some_token(TokenType::DInc, "+"),
                pos_in_chars: 3,
            },
            TokenInfo {
                token: some_token(TokenType::DDec, "-"),
                pos_in_chars: 5,
            },
            TokenInfo {
                token: some_token(TokenType::DInc, "+"),
                pos_in_chars: 7,
            },
            // head of UntileZero(...)
            TokenInfo {
                token: some_token(TokenType::LoopHead, "["),
                pos_in_chars: 11,
            },
            // ... Input
            TokenInfo {
                token: some_token(TokenType::Input, ","),
                pos_in_chars: 13,
            },
            // ... Output
            TokenInfo {
                token: some_token(TokenType::Output, "."),
                pos_in_chars: 17,
            },
            // ... Nop (PInc/PDec)
            TokenInfo {
                token: some_token(TokenType::PDec, ">"),
                pos_in_chars: 19,
            },
            TokenInfo {
                token: some_token(TokenType::PInc, "<"),
                pos_in_chars: 23,
            },
            // ... Nop (DInc/DDec)
            TokenInfo {
                token: some_token(TokenType::DDec, "+"),
                pos_in_chars: 29,
            },
            TokenInfo {
                token: some_token(TokenType::DInc, "-"),
                pos_in_chars: 31,
            },
            // tail of UntilZero
            TokenInfo {
                token: some_token(TokenType::LoopTail, "]"),
                pos_in_chars: 37,
            },
        ],
        last_pos_in_chars: 41,
    };

    const TEST_TOKENIZER_UNEXPECTED_NDO_OF_LOOP: TestTokenizer = TestTokenizer {
        tokens: &[
            // tail of UntilZero
            TokenInfo {
                token: some_token(TokenType::LoopTail, "]"),
                pos_in_chars: 1,
            },
        ],
        last_pos_in_chars: 2,
    };

    const TEST_TOKENIZER_UNEXPECTED_NDO_OF_FILE: TestTokenizer = TestTokenizer {
        tokens: &[
            // tail of UntilZero
            TokenInfo {
                token: some_token(TokenType::LoopHead, "["),
                pos_in_chars: 1,
            },
        ],
        last_pos_in_chars: 2,
    };

    // TODO: change to use assert_matches macro after the macro is stabilized.
    macro_rules! assert_matches_simple {
        ($actual:expr, $pattern:pat) => {
            match $actual {
                $pattern => {}
                _ => panic!(
                    "assertion failed: `{:?}` does not match `{:?}`",
                    $actual,
                    stringify!($pattern)
                ),
            }
        };
    }

    #[test]
    fn test_parse_str() {
        let parser = Parser::new(TEST_TOKENIZER);

        // source is dummy.
        let parsed = parser.parse_str("").expect("must be ok");

        assert_eq!(
            parsed.instructions(),
            [
                Instruction::PAdd(-1),
                Instruction::DAdd(1),
                Instruction::UntilZero(vec![Instruction::Input, Instruction::Output,],),
            ],
        );
    }

    #[test]
    fn test_parse_str_unexpected_end_of_loop() {
        let parser = Parser::new(TEST_TOKENIZER_UNEXPECTED_NDO_OF_LOOP);

        // source is dummy
        let error = parser.parse_str("").expect_err("must be err");
        assert_matches_simple!(error, ParseError::UnexpectedEndOfLoop { pos_in_chars: 1 });
    }

    #[test]
    fn test_parse_str_unexpected_end_of_file() {
        let parser = Parser::new(TEST_TOKENIZER_UNEXPECTED_NDO_OF_FILE);

        // source is dummy
        let error = parser.parse_str("").expect_err("must be err");
        assert_matches_simple!(error, ParseError::UnexpectedEndOfFile { pos_in_chars: 2 });
    }

    #[test]
    fn test_parse_str_fat() {
        let parser = Parser::new(TEST_TOKENIZER);

        // source is dummy.
        let parsed = parser.parse_str_fat("").expect("must be ok");

        assert_eq!(
            parsed.instructions(),
            [
                FatInstruction {
                    kind: FatInstructionKind::PAdd(-1),
                    tokens: vec![
                        TokenInfo {
                            token: some_token(TokenType::PInc, ">"),
                            pos_in_chars: 0,
                        },
                        TokenInfo {
                            token: some_token(TokenType::PDec, "<"),
                            pos_in_chars: 1,
                        },
                        TokenInfo {
                            token: some_token(TokenType::PDec, "<"),
                            pos_in_chars: 2,
                        },
                    ],
                },
                FatInstruction {
                    kind: FatInstructionKind::DAdd(1),
                    tokens: vec![
                        TokenInfo {
                            token: some_token(TokenType::DInc, "+"),
                            pos_in_chars: 3,
                        },
                        TokenInfo {
                            token: some_token(TokenType::DDec, "-"),
                            pos_in_chars: 5,
                        },
                        TokenInfo {
                            token: some_token(TokenType::DInc, "+"),
                            pos_in_chars: 7,
                        },
                    ],
                },
                FatInstruction {
                    kind: FatInstructionKind::UntilZero(vec![
                        FatInstruction {
                            kind: FatInstructionKind::Input,
                            tokens: vec![TokenInfo {
                                token: some_token(TokenType::Input, ","),
                                pos_in_chars: 13,
                            },],
                        },
                        FatInstruction {
                            kind: FatInstructionKind::Output,
                            tokens: vec![TokenInfo {
                                token: some_token(TokenType::Output, "."),
                                pos_in_chars: 17,
                            },],
                        },
                        FatInstruction {
                            kind: FatInstructionKind::Nop,
                            tokens: vec![
                                TokenInfo {
                                    token: some_token(TokenType::PDec, ">"),
                                    pos_in_chars: 19,
                                },
                                TokenInfo {
                                    token: some_token(TokenType::PInc, "<"),
                                    pos_in_chars: 23,
                                },
                            ],
                        },
                        FatInstruction {
                            kind: FatInstructionKind::Nop,
                            tokens: vec![
                                TokenInfo {
                                    token: some_token(TokenType::DDec, "+"),
                                    pos_in_chars: 29,
                                },
                                TokenInfo {
                                    token: some_token(TokenType::DInc, "-"),
                                    pos_in_chars: 31,
                                },
                            ],
                        }
                    ],),
                    tokens: vec![
                        TokenInfo {
                            token: some_token(TokenType::LoopHead, "["),
                            pos_in_chars: 11,
                        },
                        TokenInfo {
                            token: some_token(TokenType::Input, ","),
                            pos_in_chars: 13,
                        },
                        TokenInfo {
                            token: some_token(TokenType::Output, "."),
                            pos_in_chars: 17,
                        },
                        TokenInfo {
                            token: some_token(TokenType::PDec, ">"),
                            pos_in_chars: 19,
                        },
                        TokenInfo {
                            token: some_token(TokenType::PInc, "<"),
                            pos_in_chars: 23,
                        },
                        TokenInfo {
                            token: some_token(TokenType::DDec, "+"),
                            pos_in_chars: 29,
                        },
                        TokenInfo {
                            token: some_token(TokenType::DInc, "-"),
                            pos_in_chars: 31,
                        },
                        TokenInfo {
                            token: some_token(TokenType::LoopTail, "]"),
                            pos_in_chars: 37,
                        },
                    ],
                },
            ],
        );
    }

    #[test]
    fn test_parse_str_fat_unexpected_end_of_loop() {
        let parser = Parser::new(TEST_TOKENIZER_UNEXPECTED_NDO_OF_LOOP);

        // source is dummy
        let error = parser.parse_str_fat("").expect_err("must be err");
        assert_matches_simple!(error, ParseError::UnexpectedEndOfLoop { pos_in_chars: 1 });
    }

    #[test]
    fn test_parse_str_fat_unexpected_end_of_file() {
        let parser = Parser::new(TEST_TOKENIZER_UNEXPECTED_NDO_OF_FILE);

        // source is dummy
        let error = parser.parse_str_fat("").expect_err("must be err");
        assert_matches_simple!(error, ParseError::UnexpectedEndOfFile { pos_in_chars: 2 });
    }
}
