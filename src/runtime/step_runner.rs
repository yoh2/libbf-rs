///! Step-by-step program runner.
use crate::prelude::ProgramIndex;

use super::*;

/// A step-by-step program runner.
///
/// This runner runs the program step-by-step.
///
/// It is useful for debugging, visual representation backend and etc,...
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
    /// Create a new runner with the given inputand  output.
    pub fn new(program: &'a Program, input: R, output: W) -> Self {
        Self::with_memsize(program, input, output, DEFAULT_MEMSIZE)
    }

    /// Create a new runner with the given input, output and memory size.
    pub fn with_memsize(program: &'a Program, input: R, output: W, memsize: MemorySize) -> Self {
        let runtime = internal::Runtime::new(input, output, memsize);
        Self {
            program,
            runtime,
            index: program.first_index(),
        }
    }

    /// Get the index of the instruction to be executed.
    ///
    /// If the program is finished, this returns `None`.
    pub fn get_index(&self) -> Option<&ProgramIndex> {
        self.index.as_ref()
    }

    /// Get the current instruction to be executed.
    pub fn get_current_instruction(&self) -> Option<&Instruction> {
        self.index.as_ref().map(|index| &self.program[index])
    }

    /// Get the pointer.
    pub fn get_pointer(&self) -> isize {
        self.runtime.get_pointer()
    }

    /// Get mutable reference of data at `addres'.
    pub fn get_data_at_mut(&mut self, address: isize) -> Option<&mut u8> {
        self.runtime.get_data_at_mut(address)
    }

    /// Returns `true` if the program is running.
    pub fn is_running(&self) -> bool {
        self.index.is_some()
    }

    /// Execute the program one step.
    pub fn step(&mut self) -> Result<(), RuntimeError> {
        if let Some(index) = &mut self.index {
            let inst = &self.program[index];
            match self.runtime.exec_one(inst)? {
                internal::NextAction::Next => {
                    if !self.program.step_index(index) && !index.step_out() {
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
