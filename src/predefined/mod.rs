pub mod brainfxck {
    use crate::token::simple::{SimpleTokenSpec, SimpleTokenizer};

    pub const TOKEN_SPEC: SimpleTokenSpec<
        &'static str,
        &'static str,
        &'static str,
        &'static str,
        &'static str,
        &'static str,
        &'static str,
        &'static str,
    > = SimpleTokenSpec {
        ptr_inc: ">",
        ptr_dec: "<",
        data_inc: "+",
        data_dec: "-",
        output: ".",
        input: ",",
        loop_head: "[",
        loop_tail: "]",
    };

    pub fn tokenizer() -> SimpleTokenizer {
        TOKEN_SPEC.to_tokenizer()
    }

    #[cfg(test)]
    mod test {
        use super::*;
        use crate::{parser, runtime};

        #[test]
        fn test_hello_world() {
            let source = "++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++.";
            let program = match parser::parse_str(&tokenizer(), source) {
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
}

pub mod ook {

    use crate::{
        error::ParseError,
        token::{TokenInfo, TokenStream, TokenType, Tokenizer},
    };

    #[derive(Debug, Clone, Copy)]
    enum OokTokenType {
        /// Ook.
        OokDot,
        /// Ook?
        OokQuestion,
        /// Ook!
        OokExclamation,
        /// Eof
        Eof,
    }

    struct OokTokenInfo {
        token_type: OokTokenType,
        /// The position of the token in the source.
        pos_in_chars: usize,
    }

    pub struct OokTokenizer;

    impl<'a> Tokenizer<'a> for OokTokenizer {
        type Stream = OokTokenStream<'a>;

        fn token_stream(&'a self, source: &'a str) -> Self::Stream {
            OokTokenStream::new(source)
        }
    }

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
                        Some('.') => OokTokenType::OokDot,
                        Some('?') => OokTokenType::OokQuestion,
                        Some('!') => OokTokenType::OokExclamation,
                        _ => {
                            rel_pos_in_chars += 1;
                            continue;
                        }
                    };
                    let info = OokTokenInfo {
                        token_type,
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
                token_type: OokTokenType::Eof,
                pos_in_chars: self.pos_in_chars,
            }
        }
    }

    impl<'a> TokenStream for OokTokenStream<'a> {
        fn next(&mut self) -> Result<TokenInfo, ParseError> {
            let first_token = self.next_ook_token();
            if let OokTokenType::Eof = first_token.token_type {
                return Ok(TokenInfo {
                    token_type: TokenType::Eof,
                    pos_in_chars: first_token.pos_in_chars,
                });
            }

            let second_token = self.next_ook_token();
            if let OokTokenType::Eof = second_token.token_type {
                return Err(ParseError::MiscError(
                    second_token.pos_in_chars,
                    "Odd number of Ook tokens".to_string(),
                ));
            }

            let token_type = match (first_token.token_type, second_token.token_type) {
                (OokTokenType::OokDot, OokTokenType::OokQuestion) => TokenType::PInc,
                (OokTokenType::OokQuestion, OokTokenType::OokDot) => TokenType::PDec,
                (OokTokenType::OokDot, OokTokenType::OokDot) => TokenType::DInc,
                (OokTokenType::OokExclamation, OokTokenType::OokExclamation) => TokenType::DDec,
                (OokTokenType::OokExclamation, OokTokenType::OokDot) => TokenType::Output,
                (OokTokenType::OokDot, OokTokenType::OokExclamation) => TokenType::Input,
                (OokTokenType::OokExclamation, OokTokenType::OokQuestion) => TokenType::LoopHead,
                (OokTokenType::OokQuestion, OokTokenType::OokExclamation) => TokenType::LoopTail,

                (OokTokenType::OokQuestion, OokTokenType::OokQuestion) => {
                    return Err(ParseError::MiscError(
                        first_token.pos_in_chars,
                        "Ook? Ook?: bad Ook sequence".to_string(),
                    ))
                }
                _ => unreachable!(),
            };

            Ok(TokenInfo {
                token_type,
                pos_in_chars: first_token.pos_in_chars,
            })
        }
    }

    #[cfg(test)]
    mod test {
        use super::*;
        use crate::{parser, runtime};

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
            let program = match parser::parse_str(&OokTokenizer, source) {
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
            if let Err(err) = parser::parse_str(&OokTokenizer, source) {
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
            if let Err(err) = parser::parse_str(&OokTokenizer, source) {
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
}
