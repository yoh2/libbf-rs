use super::*;

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

pub enum NextAction<'a> {
    Next,
    StepIn(&'a [Instruction]),
}

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
    pub fn new(input: R, output: W, memsize: MemorySize) -> Self {
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
        if self.input.read(std::slice::from_mut(data))? == 0 {
            Err(RuntimeError::Eof)
        } else {
            Ok(())
        }
    }

    fn output(&mut self) -> Result<(), RuntimeError> {
        let data = self.memory.get_mut(self.pointer)?;
        self.output.write_all(std::slice::from_ref(data))?;
        Ok(())
    }

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

    // the following methods are for Brainf*ck program debugging.

    pub fn get_pointer(&self) -> isize {
        self.pointer
    }

    /// Get the memory data at the address.
    ///
    /// Returns `None` if the address is out of memory bounds.
    pub fn get_data_at_mut(&mut self, address: isize) -> Option<&mut u8> {
        self.memory.get_mut(address).ok()
    }
}
