use std::io::Read;

use crate::{
    error::{ParseError, ParseOrIoError},
    program::{Instruction, Program},
    token::{TokenInfo, TokenStream, TokenType, Tokenizer},
};

struct ParseContext<T> {
    token_stream: T,

    // (length, char count, token type) of ungot token
    unget_buf: Option<TokenInfo>,
}

impl<T> ParseContext<T>
where
    T: TokenStream,
{
    fn new(token_stream: T) -> Self {
        Self {
            token_stream,
            unget_buf: None,
        }
    }

    fn next_token_info(&mut self) -> Result<TokenInfo, ParseError> {
        if let Some(def) = self.unget_buf.take() {
            return Ok(def);
        }
        self.token_stream.next()
    }

    fn unget_token_info(&mut self, info: TokenInfo) {
        assert!(self.unget_buf.is_none());
        self.unget_buf = Some(info);
    }

    fn parse(&mut self, top_level: bool) -> Result<Program, ParseError> {
        let mut program = Program::new();

        loop {
            let info = self.next_token_info()?;
            match info.token_type {
                TokenType::PInc => self.push_padd(&mut program, 1)?,
                TokenType::PDec => self.push_padd(&mut program, -1)?,
                TokenType::DInc => self.push_dadd(&mut program, 1)?,
                TokenType::DDec => self.push_dadd(&mut program, -1)?,
                TokenType::Output => program.push(Instruction::Output),
                TokenType::Input => program.push(Instruction::Input),
                TokenType::LoopHead => program.push(Instruction::UntilZero(self.parse(false)?)),
                TokenType::LoopTail => {
                    if top_level {
                        return Err(ParseError::UnexpectedEndOfLoop(info.pos_in_chars));
                    } else {
                        return Ok(program);
                    }
                }

                TokenType::Eof => {
                    return if top_level {
                        Ok(program)
                    } else {
                        Err(ParseError::UnexpectedEndOfFile(info.pos_in_chars))
                    }
                }
            }
        }
    }

    fn push_padd(
        &mut self,
        program: &mut Program,
        initial_operand: isize,
    ) -> Result<(), ParseError> {
        self.push_xadd(
            program,
            initial_operand,
            TokenType::PInc,
            TokenType::PDec,
            Instruction::PAdd,
        )
    }

    fn push_dadd(
        &mut self,
        program: &mut Program,
        initial_operand: isize,
    ) -> Result<(), ParseError> {
        self.push_xadd(
            program,
            initial_operand,
            TokenType::DInc,
            TokenType::DDec,
            Instruction::DAdd,
        )
    }

    fn push_xadd(
        &mut self,
        program: &mut Program,
        initial_operand: isize,
        inc: TokenType,
        dec: TokenType,
        gen: fn(isize) -> Instruction,
    ) -> Result<(), ParseError> {
        let mut operand = initial_operand;

        loop {
            let info = self.next_token_info()?;
            if info.token_type == inc {
                operand += 1;
            } else if info.token_type == dec {
                operand -= 1;
            } else {
                self.unget_token_info(info);
                break;
            }
        }

        if operand != 0 {
            program.push(gen(operand));
        }
        Ok(())
    }
}

pub fn parse(
    tokenizer: &impl for<'x> Tokenizer<'x>,
    mut reader: impl Read,
) -> Result<Program, ParseOrIoError> {
    let mut source = String::new();
    let _ = reader.read_to_string(&mut source)?;
    parse_str(tokenizer, &source).map_err(ParseOrIoError::ParseError)
}

pub fn parse_str(
    tokenizer: &impl for<'x> Tokenizer<'x>,
    source: &str,
) -> Result<Program, ParseError> {
    let mut context = ParseContext::new(tokenizer.token_stream(source));
    context.parse(true)
}
