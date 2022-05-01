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
