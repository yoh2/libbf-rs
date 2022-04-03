# libbf-rs
Brainf*ck-like language library wihch is written in Rust

## Examples

### Use predefined Brainf*ck interpreter

`brainfxck' feature flag is needed to compile this example.

```rust
use libbf::{predefined::brainfxck, runtime};
use std::io;

fn main() {
    let source = "++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++.";
    let program = brainfxck::parser()
        .parse_str(source)
        .expect("Failed to parse");
    runtime::run(&program, io::stdin(), io::stdout()).expect("Failed to run");
}
```

### Define Brainf*ck interpreter

```rust
use libbf::{parser::Parser, runtime, token::simple::SimpleTokenSpec};
use std::io;

fn main() {
    // Create parser with token specification.
    let parser = Parser::new(
        SimpleTokenSpec {
            // You can write tokens with `ToString` (`char`, `&str`, `String`, etc.)
            ptr_inc: '>',              // char
            ptr_dec: "<",              // &str
            data_inc: ".".to_string(), // String
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
 - `predefined` - predefined interpreters below
 - `braninfxck` - pre-defined brainf*ck interpreter
 - `ook` - pre-defined Ook! interpreter

## TODO

Write documentations and tests.