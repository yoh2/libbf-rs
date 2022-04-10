//! Brainfuck-like language library.
//!
//! This library can define a variant of Brainfuck-like language parser
//! and can run parsed program.
//!
//! # Examples
//!
//! ```
//! use libbf::{parser::Parser, runtime, token::simple::SimpleTokenSpec};
//! use std::io::{self, Read};
//!
//! // Create parser with token specification.
//! let parser = Parser::new(
//!     SimpleTokenSpec {
//!         // You can specify tokens with `ToString` (`char`, `&str`, `String`, etc.)
//!         ptr_inc: '>',              // char
//!         ptr_dec: "<",              // &str
//!         data_inc: "+".to_string(), // String
//!         data_dec: '-',
//!         output: '.',
//!         input: ',',
//!         loop_head: '[',
//!         loop_tail: ']',
//!     }
//!     .to_tokenizer(),
//! );
//!
//! let source = "++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++.";
//! let program = parser.parse_str(source).expect("Failed to parse");
//! let mut output = Vec::new();
//! let result = runtime::run(&program, io::stdin(), &mut output);
//!
//! assert!(result.is_ok());
//! assert_eq!(output, b"Hello World!\n");
//! ```
pub mod error;
pub mod parser;
#[cfg(any(feature = "brainfxck", feature = "ook"))]
pub mod predefined;
pub mod program;
pub mod runtime;
pub mod token;

/// `use libbf::prelude::*` is easy way to use this library;
pub mod prelude {
    pub use crate::error::*;
    pub use crate::parser::*;
    #[cfg(feature = "brainfxck")]
    pub use crate::predefined::brainfxck;
    #[cfg(feature = "ook")]
    pub use crate::predefined::ook;
    pub use crate::program::*;
    // exclude functions in runtime::*
    pub use crate::runtime::{self, MemorySize, Runner, StepRunner, DEFAULT_MEMSIZE};
    pub use crate::token::simple::*;
    pub use crate::token::*;
}
