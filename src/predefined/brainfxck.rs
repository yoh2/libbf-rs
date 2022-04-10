///! Predefined Brainf*ck implementations.
use crate::{
    prelude::Parser,
    token::simple::{SimpleTokenSpec1, SimpleTokenizer},
};

/// A token specification for Brainf*ck.
pub const TOKEN_SPEC: SimpleTokenSpec1<char> = SimpleTokenSpec1 {
    ptr_inc: '>',
    ptr_dec: '<',
    data_inc: '+',
    data_dec: '-',
    output: '.',
    input: ',',
    loop_head: '[',
    loop_tail: ']',
};

/// Create a tokenizer for Brainf*ck.
///
/// This is equivalent to call of `TOKEN_SPEC.to_tokenizer()`
pub fn tokenizer() -> SimpleTokenizer {
    TOKEN_SPEC.to_tokenizer()
}

/// Create a parser for Brainf*ck.
///
/// This is equivalent to call of `Parser::new(tokenizer())`
pub fn parser() -> Parser<SimpleTokenizer> {
    Parser::new(tokenizer())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::runtime;

    #[test]
    fn test_hello_world() {
        let source = "++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++.";
        let program = match parser().parse_str(source) {
            Ok(program) => program,
            Err(err) => panic!("unexpected error: {err}"),
        };

        let input: &[u8] = &[];
        let mut output = vec![];
        if let Err(err) = runtime::run(&program, input, &mut output) {
            panic!("unexpected error: {err}");
        }
        assert_eq!(output, b"Hello World!\n");
    }
}
