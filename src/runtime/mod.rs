use crate::{
    error::RuntimeError,
    program::{Instruction, Program},
};

use std::io::{Read, Write};

/// Memory size
#[derive(Debug, Clone, Copy)]
pub enum MemorySize {
    /// Fixed size [0..usize). Access to memory out of bounds will cause runtime error.
    Fixed(usize),
    /// Inifinite to right (positive) direction. Access to negative address will cause runtime error.
    RightInfinite,
    /// Infinite to both (positive and negative) directions
    BothInfinite,
}

pub struct Memory {
    size: MemorySize,
    /// memory data for [0..]
    right_data: Vec<u8>,
    /// memory data for [..-1]
    left_data: Vec<u8>,
}

impl Memory {
    fn new(size: MemorySize) -> Self {
        let right_data = if let MemorySize::Fixed(len) = size {
            if len > isize::MAX as usize {
                panic!("memory size larger han isize::MAX is not supported.");
            }
            vec![0; len]
        } else {
            vec![]
        };
        let left_data = vec![];

        Self {
            size,
            right_data,
            left_data,
        }
    }

    fn get_mut(&mut self, address: isize) -> Result<&mut u8, RuntimeError> {
        if address >= 0 {
            if (address as usize) >= self.right_data.len() {
                if let MemorySize::Fixed(_) = self.size {
                    return Err(RuntimeError::OutOfMemoryBounds(address));
                }
                self.right_data.resize(address as usize + 1, 0);
            }
            Ok(&mut self.right_data[address as usize])
        } else if let MemorySize::BothInfinite = self.size {
            let left_address = (-(address + 1)) as usize;
            if left_address >= self.left_data.len() {
                self.left_data.resize(left_address + 1, 0);
            }
            Ok(&mut self.left_data[left_address])
        } else {
            Err(RuntimeError::OutOfMemoryBounds(address))
        }
    }
}

struct Runtime<R, W> {
    input: R,
    output: W,
    memory: Memory,
    pointer: isize,
}

impl<R, W> Runtime<R, W>
where
    R: Read,
    W: Write,
{
    fn new(input: R, output: W, memsize: MemorySize) -> Self {
        Self {
            input,
            output,
            memory: Memory::new(memsize),
            pointer: 0,
        }
    }

    fn add_pointer(&mut self, operand: isize) -> Result<(), RuntimeError> {
        self.pointer += operand;
        Ok(())
    }

    fn add_data(&mut self, operand: isize) -> Result<(), RuntimeError> {
        let data = self.memory.get_mut(self.pointer)?;
        *data = (*data as isize).wrapping_add(operand) as u8;
        Ok(())
    }

    fn input(&mut self) -> Result<(), RuntimeError> {
        let data = self.memory.get_mut(self.pointer)?;
        self.input.read_exact(std::slice::from_mut(data))?;
        Ok(())
    }

    fn output(&mut self) -> Result<(), RuntimeError> {
        let data = self.memory.get_mut(self.pointer)?;
        self.output.write_all(std::slice::from_ref(data))?;
        Ok(())
    }

    // this method cannot call more than once on the same instance
    fn run(&mut self, program: &Program) -> Result<(), RuntimeError> {
        for inst in program {
            match inst {
                Instruction::PAdd(operand) => self.add_pointer(*operand)?,
                Instruction::DAdd(operand) => self.add_data(*operand)?,
                Instruction::Output => self.output()?,
                Instruction::Input => self.input()?,
                Instruction::UntilZero(sub) => {
                    while *self.memory.get_mut(self.pointer)? != 0 {
                        self.run(sub)?;
                    }
                }
            }
        }

        Ok(())
    }
}

pub const DEFAULT_MEMSIZE: MemorySize = MemorySize::Fixed(30000);

pub fn run<R, W>(program: &Program, input: R, output: W) -> Result<(), RuntimeError>
where
    R: Read,
    W: Write,
{
    run_with_memsize(program, input, output, DEFAULT_MEMSIZE)
}

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
    let mut runtime = Runtime::new(input, output, memsize);
    runtime.run(program)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_run_empty_program() {
        let program = vec![];
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
        let program = vec![Input, Output, Input, Output];
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
        let program = vec![Input, DAdd(3), Output];
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
        let program = vec![PAdd(-1), DAdd(1)];
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
            assert!(false, "unexpectedly succeeded");
        }
    }

    #[test]
    fn test_run_out_of_memory_bounds_left_for_right_infinity() {
        use Instruction::*;
        let program = vec![PAdd(-1), DAdd(1)];
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
            assert!(false, "unexpectedly succeeded");
        }
    }

    #[test]
    fn test_run_negative_memory_address_access() {
        use Instruction::*;
        let program = vec![PAdd(-1), DAdd(1)];
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
        let program = vec![PAdd(30000), DAdd(1)];
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
            assert!(false, "unexpectedly succeeded");
        }
    }

    #[test]
    fn test_run_positive_memory_address_access_for_right_inifinite() {
        use Instruction::*;
        let program = vec![PAdd(65536), DAdd(1)];
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
        let program = vec![PAdd(65536), DAdd(1)];
        let input: &[u8] = &[];
        let mut output = vec![];
        let result = run_with_memsize(&program, input, &mut output, MemorySize::BothInfinite);
        if let Err(e) = result {
            panic!("unexpected error: {e}");
        }
    }

    #[test]
    fn test_run_input_error() {
        use Instruction::*;
        let program = vec![Input];
        let input: &[u8] = &[];
        let mut output = vec![];
        let result = run(&program, input, &mut output);
        if let Err(e) = result {
            if let RuntimeError::IoError(_) = e {
                // OK
            } else {
                panic!("unexpected error: {e}");
            }
        } else {
            assert!(false, "unexpectedly succeeded");
        }
    }

    #[test]
    fn test_run_output_error() {
        use Instruction::*;
        let program = vec![Output];
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
            assert!(false, "unexpectedly succeeded");
        }
    }

    #[test]
    fn test_run_hello_world() {
        use Instruction::*;
        // Hello World
        let program = vec![
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
        ];
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
