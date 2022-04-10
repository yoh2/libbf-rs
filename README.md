# libbf

Brainfuck-like language library.

This library can define a variant of Brainfuck-like language parser
and can run parsed program.

## Examples

### Use predefined Brainfuck interpreter

`bf' feature flag is needed to compile this example.

```rust
use libbf::{predefined::bf, runtime};
use std::io;

fn main() {
    let source = "++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++.";
    let program = bf::parser()
        .parse_str(source)
        .expect("Failed to parse");
    runtime::run(&program, io::stdin(), io::stdout()).expect("Failed to run");
}
```

### Define Brainfuck interpreter

```rust
use libbf::{parser::Parser, runtime, token::simple::SimpleTokenSpec};
use std::io;

fn main() {
    // Create parser with token specification.
    let parser = Parser::new(
        SimpleTokenSpec {
            // You can specify tokens with `ToString` (`char`, `&str`, `String`, etc.)
            ptr_inc: '>',              // char
            ptr_dec: "<",              // &str
            data_inc: "+".to_string(), // String
            data_dec: '-',
            output: '.',
            input: ',',
            loop_head: '[',
            loop_tail: ']',
        }
        .to_tokenizer(),
    );

    let source = "++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++.";
    let program = parser.parse_str(source).expect("Failed to parse");
    runtime::run(&program, io::stdin(), io::stdout()).expect("Failed to run");
}
```

### Feature flags

 - `all` - all features
 - `predefined` - predefined parsers below
 - `bf` - predefined Brainfuck parser
 - `ook` - predefined Ook! parser
