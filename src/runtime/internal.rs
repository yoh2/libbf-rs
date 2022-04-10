use super::*;

/// A runtime memory.
pub struct Memory {
    size: MemorySize,
    /// memory data for [0..]
    right_data: Vec<u8>,
    /// memory data for [..-1]
    left_data: Vec<u8>,
}

impl Memory {
    /// Creates a new memory with the given size.
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

    /// Get the mutable reference of the memory data at the given address.
    ///
    /// If the address is out of range, this function returns error [`RuntimeError::OutOfMemoryBounds`].
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

/// Next of
pub enum NextAction<'a> {
    Next,
    StepIn(&'a [Instruction]),
}

/// A program runtime.
pub struct Runtime<R, W> {
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
    /// Create a new runtime with the given input, output and memory size.
    pub fn new(input: R, output: W, memsize: MemorySize) -> Self {
        Self {
            input,
            output,
            memory: Memory::new(memsize),
            pointer: 0,
        }
    }

    // Add operand to the pointer.
    fn add_pointer(&mut self, operand: isize) -> Result<(), RuntimeError> {
        self.pointer += operand;
        Ok(())
    }

    // Add operand to the data which is pointed by the pointer.
    fn add_data(&mut self, operand: isize) -> Result<(), RuntimeError> {
        let data = self.memory.get_mut(self.pointer)?;
        *data = (*data as isize).wrapping_add(operand) as u8;
        Ok(())
    }

    // Read a byte from the input and store it to the data which is pointed by the pointer.
    fn input(&mut self) -> Result<(), RuntimeError> {
        let data = self.memory.get_mut(self.pointer)?;
        if self.input.read(std::slice::from_mut(data))? == 0 {
            Err(RuntimeError::Eof)
        } else {
            Ok(())
        }
    }

    // Write a byte which is pointed by the pointer to the output.
    fn output(&mut self) -> Result<(), RuntimeError> {
        let data = self.memory.get_mut(self.pointer)?;
        self.output.write_all(std::slice::from_ref(data))?;
        Ok(())
    }

    /// Execute specified instruction and return a next action to be performed.
    ///
    /// If `inst` is [`Instruction::UntilZero`] and the data which is pointed by the pointer is not zero,
    /// this function returns [`NextAction::StepIn`] with instructions that `inst` has.
    ///
    /// If `inst` is [`Instruction::UntilZero`] and the data which is pointed by the pointer is zero or
    /// `inst` is other instruction, this function returns [`NextAction::Next`].
    ///
    /// In any case, if an error occurred, this function returns that error.
    pub fn exec_one<'a>(&mut self, inst: &'a Instruction) -> Result<NextAction<'a>, RuntimeError> {
        match inst {
            Instruction::PAdd(operand) => self.add_pointer(*operand)?,
            Instruction::DAdd(operand) => self.add_data(*operand)?,
            Instruction::Output => self.output()?,
            Instruction::Input => self.input()?,
            Instruction::UntilZero(sub) => {
                if *self.memory.get_mut(self.pointer)? != 0 {
                    return Ok(NextAction::StepIn(sub));
                }
            }
        }
        Ok(NextAction::Next)
    }

    // the following methods are for Brainfuck program debugging.

    /// Get the pointer of the runtime.
    pub fn get_pointer(&self) -> isize {
        self.pointer
    }

    /// Get the memory data which is pointed by the pointer.
    ///
    /// Returns `None` if the address is out of memory bounds.
    pub fn get_data_at_mut(&mut self, address: isize) -> Option<&mut u8> {
        self.memory.get_mut(address).ok()
    }
}
