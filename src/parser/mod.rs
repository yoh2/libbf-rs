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

    fn parse(&mut self, top_level: bool) -> Result<Vec<Instruction>, ParseError> {
        let mut instructions = Vec::new();

        loop {
            let info = self.next_token_info()?;
            match info.token_type {
                Some(TokenType::PInc) => self.push_padd(&mut instructions, 1)?,
                Some(TokenType::PDec) => self.push_padd(&mut instructions, -1)?,
                Some(TokenType::DInc) => self.push_dadd(&mut instructions, 1)?,
                Some(TokenType::DDec) => self.push_dadd(&mut instructions, -1)?,
                Some(TokenType::Output) => instructions.push(Instruction::Output),
                Some(TokenType::Input) => instructions.push(Instruction::Input),
                Some(TokenType::LoopHead) => {
                    instructions.push(Instruction::UntilZero(self.parse(false)?))
                }
                Some(TokenType::LoopTail) => {
                    if top_level {
                        return Err(ParseError::UnexpectedEndOfLoop(info.pos_in_chars));
                    } else {
                        return Ok(instructions);
                    }
                }

                None => {
                    return if top_level {
                        Ok(instructions)
                    } else {
                        Err(ParseError::UnexpectedEndOfFile(info.pos_in_chars))
                    }
                }
            }
        }
    }

    fn push_padd(
        &mut self,
        instructions: &mut Vec<Instruction>,
        initial_operand: isize,
    ) -> Result<(), ParseError> {
        self.push_xadd(
            instructions,
            initial_operand,
            TokenType::PInc,
            TokenType::PDec,
            Instruction::PAdd,
        )
    }

    fn push_dadd(
        &mut self,
        instructions: &mut Vec<Instruction>,
        initial_operand: isize,
    ) -> Result<(), ParseError> {
        self.push_xadd(
            instructions,
            initial_operand,
            TokenType::DInc,
            TokenType::DDec,
            Instruction::DAdd,
        )
    }

    fn push_xadd(
        &mut self,
        instructions: &mut Vec<Instruction>,
        initial_operand: isize,
        inc: TokenType,
        dec: TokenType,
        gen: fn(isize) -> Instruction,
    ) -> Result<(), ParseError> {
        let mut operand = initial_operand;

        loop {
            let info = self.next_token_info()?;
            if info.token_type == Some(inc) {
                operand += 1;
            } else if info.token_type == Some(dec) {
                operand -= 1;
            } else {
                // unget token other than inc or dec (including EOF.)
                self.unget_token_info(info);
                break;
            }
        }

        if operand != 0 {
            instructions.push(gen(operand));
        }
        Ok(())
    }
}

pub struct Parser<T> {
    tokenizer: T,
}

impl<T> Parser<T>
where
    for<'x> T: Tokenizer<'x>,
{
    pub fn new(tokenizer: T) -> Self {
        Self { tokenizer }
    }

    pub fn parse(&self, mut reader: impl Read) -> Result<Program, ParseOrIoError> {
        let mut source = String::new();
        let _ = reader.read_to_string(&mut source)?;
        let program = self.parse_str(&source)?;
        Ok(program)
    }

    pub fn parse_str(&self, source: &str) -> Result<Program, ParseError> {
        let mut context = ParseContext::new(self.tokenizer.token_stream(source));
        Ok(Program::new(context.parse(true)?))
    }
}
