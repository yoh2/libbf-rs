use crate::prelude::ProgramIndex;

use super::*;

pub struct StepRunner<'a, R, W> {
    program: &'a Program,
    runtime: internal::Runtime<R, W>,
    index: Option<ProgramIndex>,
}

impl<'a, R, W> StepRunner<'a, R, W>
where
    R: Read,
    W: Write,
{
    pub fn new(program: &'a Program, input: R, output: W) -> Self {
        Self::with_memsize(program, input, output, DEFAULT_MEMSIZE)
    }

    pub fn with_memsize(program: &'a Program, input: R, output: W, memsize: MemorySize) -> Self {
        let runtime = internal::Runtime::new(input, output, memsize);
        Self {
            program,
            runtime,
            index: program.first_index(),
        }
    }

    pub fn get_index(&self) -> Option<&ProgramIndex> {
        self.index.as_ref()
    }

    pub fn get_current_instruction(&self) -> Option<&Instruction> {
        self.index.as_ref().map(|index| &self.program[index])
    }

    pub fn get_pointer(&self) -> isize {
        self.runtime.get_pointer()
    }

    pub fn get_data_at_mut(&mut self, address: isize) -> Option<&mut u8> {
        self.runtime.get_data_at_mut(address)
    }

    pub fn is_running(&self) -> bool {
        self.index.is_some()
    }

    pub fn step(&mut self) -> Result<(), RuntimeError> {
        if let Some(index) = &mut self.index {
            let inst = &self.program[index];
            match self.runtime.exec_one(inst)? {
                internal::NextAction::Next => {
                    if !self.program.step_index(index) && index.step_out() {
                        self.index = None;
                    }
                }
                internal::NextAction::StepIn(sub) => {
                    if !sub.is_empty() {
                        index.step_in();
                    }
                }
            }
        }
        Ok(())
    }
}
