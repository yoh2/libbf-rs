use super::{TokenInfo, TokenStream, TokenType, Tokenizer};

/// Token specification for `SimpleTokenizer`.
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

/// A variant of `SimpleTokenSpec` where all members have the same type.
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

// Token definition
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

pub struct SimpleTokenizer {
    token_table: Vec<SimpleTokenDef>,
}

impl<'a> Tokenizer<'a> for SimpleTokenizer {
    type Stream = SimpleTokenStream<'a>;

    fn token_stream(&'a self, source: &'a str) -> SimpleTokenStream<'a> {
        SimpleTokenStream::new(source, &self.token_table)
    }
}

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

impl<'a> TokenStream for SimpleTokenStream<'a> {
    fn next(&mut self) -> Result<TokenInfo, crate::error::ParseError> {
        // TODO: This loop is too dumb. It should use more efficient algorithm.

        let mut rel_pos_in_chars = 0;
        for (rel_pos, _) in self.source[self.pos..].char_indices() {
            if let Some(def) = find_token_at(self.source, self.pos + rel_pos, self.token_table) {
                let info = TokenInfo {
                    token_type: def.token_type,
                    pos_in_chars: self.pos_in_chars + rel_pos_in_chars,
                };
                // next position
                self.pos += rel_pos + def.token.len();
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
            token_type: TokenType::Eof,
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
