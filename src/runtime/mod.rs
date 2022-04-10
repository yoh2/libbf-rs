//! Program runtime.
mod internal;
mod runner;
mod step_runner;

use crate::{error::RuntimeError, prelude::Program, program::Instruction};

use std::io::{Read, Write};

pub use self::runner::Runner;
pub use self::step_runner::StepRunner;

/// A runtime memory size.
#[derive(Debug, Clone, Copy)]
pub enum MemorySize {
    /// Fixed size (range: [0, self.0)). Access to memory out of bounds will cause runtime error.
    Fixed(usize),
    /// Inifinite to right (positive) direction. Access to negative address will cause runtime error.
    RightInfinite,
    /// Infinite to both (positive and negative) directions
    BothInfinite,
}

/// Default memory size.
pub const DEFAULT_MEMSIZE: MemorySize = MemorySize::Fixed(30000);

/// Run a program with the given input and output.
///
/// It is equivalent to `Runner::new(input, output).run()`.
pub fn run<R, W>(program: &Program, input: R, output: W) -> Result<(), RuntimeError>
where
    R: Read,
    W: Write,
{
    Runner::new(program, input, output).run()
}

/// Run a program with the given input, output and memory size.
///
/// It is equivalent to `Runner::with_memsize(input, output, memsize).run()`.
pub fn run_with_memsize<R, W>(
    program: &Program,
    input: R,
    output: W,
    memsize: MemorySize,
) -> Result<(), RuntimeError>
where
    R: Read,
    W: Write,
{
    Runner::with_memsize(program, input, output, memsize).run()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_run_empty_program() {
        let program = Program::new([]);
        let input: &[u8] = &[];
        let mut output = vec![];
        let result = run(&program, input, &mut output);
        if let Err(e) = result {
            panic!("unexpected error: {e}");
        }
    }

    #[test]
    fn test_run_input_output() {
        use Instruction::*;
        let program = Program::new([Input, Output, Input, Output]);
        let input: &[u8] = &[42, 53];
        let mut output = vec![];
        let result = run(&program, input, &mut output);
        if let Err(e) = result {
            panic!("unexpected error: {e}");
        } else {
            assert_eq!(output, &[42, 53]);
        }
    }

    #[test]
    fn test_run_dadd_overflow() {
        use Instruction::*;
        let program = Program::new([Input, DAdd(3), Output]);
        let input: &[u8] = &[254];
        let mut output = vec![];
        let result = run(&program, input, &mut output);
        if let Err(e) = result {
            panic!("unexpected error: {e}");
        } else {
            assert_eq!(output, &[1]);
        }
    }

    #[test]
    fn test_run_out_of_memory_bounds_left() {
        use Instruction::*;
        let program = Program::new([PAdd(-1), DAdd(1)]);
        let input: &[u8] = &[];
        let mut output = vec![];
        let result = run(&program, input, &mut output);
        if let Err(e) = result {
            if let RuntimeError::OutOfMemoryBounds(pointer) = e {
                assert_eq!(pointer, -1);
            } else {
                panic!("unexpected error: {e}");
            }
        } else {
            panic!("unexpectedly succeeded");
        }
    }

    #[test]
    fn test_run_out_of_memory_bounds_left_for_right_infinity() {
        use Instruction::*;
        let program = Program::new([PAdd(-1), DAdd(1)]);
        let input: &[u8] = &[];
        let mut output = vec![];
        let result = run_with_memsize(&program, input, &mut output, MemorySize::RightInfinite);
        if let Err(e) = result {
            if let RuntimeError::OutOfMemoryBounds(pointer) = e {
                assert_eq!(pointer, -1);
            } else {
                panic!("unexpected error: {e}");
            }
        } else {
            panic!("unexpectedly succeeded");
        }
    }

    #[test]
    fn test_run_negative_memory_address_access() {
        use Instruction::*;
        let program = Program::new([PAdd(-1), DAdd(1)]);
        let input: &[u8] = &[];
        let mut output = vec![];
        let result = run_with_memsize(&program, input, &mut output, MemorySize::BothInfinite);
        if let Err(e) = result {
            panic!("unexpected error: {e}");
        }
    }

    #[test]
    fn test_run_out_of_memory_bounds_right() {
        use Instruction::*;
        let program = Program::new([PAdd(30000), DAdd(1)]);
        let input: &[u8] = &[];
        let mut output = vec![];
        let result = run(&program, input, &mut output);
        if let Err(e) = result {
            if let RuntimeError::OutOfMemoryBounds(pointer) = e {
                assert_eq!(pointer, 30000);
            } else {
                panic!("unexpected error: {e}");
            }
        } else {
            panic!("unexpectedly succeeded");
        }
    }

    #[test]
    fn test_run_positive_memory_address_access_for_right_inifinite() {
        use Instruction::*;
        let program = Program::new([PAdd(65536), DAdd(1)]);
        let input: &[u8] = &[];
        let mut output = vec![];
        let result = run_with_memsize(&program, input, &mut output, MemorySize::RightInfinite);
        if let Err(e) = result {
            panic!("unexpected error: {e}");
        }
    }

    #[test]
    fn test_run_positive_memory_address_access_for_both_inifinite() {
        use Instruction::*;
        let program = Program::new([PAdd(65536), DAdd(1)]);
        let input: &[u8] = &[];
        let mut output = vec![];
        let result = run_with_memsize(&program, input, &mut output, MemorySize::BothInfinite);
        if let Err(e) = result {
            panic!("unexpected error: {e}");
        }
    }

    struct TestErrorReader;

    impl Read for TestErrorReader {
        fn read(&mut self, _buf: &mut [u8]) -> Result<usize, std::io::Error> {
            Err(std::io::Error::new(std::io::ErrorKind::Other, "test error"))
        }
    }

    #[test]
    fn test_run_input_error() {
        use Instruction::*;
        let program = Program::new([Input]);
        let input = TestErrorReader;
        let mut output = vec![];
        let result = run(&program, input, &mut output);
        if let Err(e) = result {
            if let RuntimeError::IoError(_) = e {
                // OK
            } else {
                panic!("unexpected error: {e}");
            }
        } else {
            panic!("unexpectedly succeeded");
        }
    }

    #[test]
    fn test_run_input_eof() {
        use Instruction::*;
        let program = Program::new([Input]);
        let input: &[u8] = &[];
        let mut output = vec![];
        let result = run(&program, input, &mut output);
        if let Err(e) = result {
            if let RuntimeError::Eof = e {
                // OK
            } else {
                panic!("unexpected error: {e}");
            }
        } else {
            panic!("unexpectedly succeeded");
        }
    }

    #[test]
    fn test_run_output_error() {
        use Instruction::*;
        let program = Program::new([Output]);
        let input: &[u8] = &[];
        let mut output: &mut [u8] = &mut [];
        let result = run(&program, input, &mut output);
        if let Err(e) = result {
            if let RuntimeError::IoError(_) = e {
                // OK
            } else {
                panic!("unexpected error: {e}");
            }
        } else {
            panic!("unexpectedly succeeded");
        }
    }

    #[test]
    fn test_run_hello_world() {
        use Instruction::*;
        // Hello World
        let program = Program::new([
            DAdd(8),
            UntilZero(vec![
                PAdd(1),
                DAdd(4),
                UntilZero(vec![
                    PAdd(1),
                    DAdd(2),
                    PAdd(1),
                    DAdd(3),
                    PAdd(1),
                    DAdd(3),
                    PAdd(1),
                    DAdd(1),
                    PAdd(-4),
                    DAdd(-1),
                ]),
                PAdd(1),
                DAdd(1),
                PAdd(1),
                DAdd(1),
                PAdd(1),
                DAdd(-1),
                PAdd(2),
                DAdd(1),
                UntilZero(vec![PAdd(-1)]),
                PAdd(-1),
                DAdd(-1),
            ]),
            PAdd(2),
            Output,
            PAdd(1),
            DAdd(-3),
            Output,
            DAdd(7),
            Output,
            Output,
            DAdd(3),
            Output,
            PAdd(2),
            Output,
            PAdd(-1),
            DAdd(-1),
            Output,
            PAdd(-1),
            Output,
            DAdd(3),
            Output,
            DAdd(-6),
            Output,
            DAdd(-8),
            Output,
            PAdd(2),
            DAdd(1),
            Output,
            PAdd(1),
            DAdd(2),
            Output,
        ]);
        let input: &[u8] = &[];
        let mut output = vec![];
        let result = run(&program, input, &mut output);
        if let Err(e) = result {
            panic!("unexpected error: {e}");
        } else {
            assert_eq!(output, b"Hello World!\n");
        }
    }
}
