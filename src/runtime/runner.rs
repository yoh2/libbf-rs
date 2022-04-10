///! Basic program runner.
use super::internal::NextAction;
use super::*;

/// A basic program runner.
///
/// This runner runs the entire program at once.
pub struct Runner<'a, R, W> {
    program: &'a Program,
    runtime: internal::Runtime<R, W>,
}

impl<'a, R, W> Runner<'a, R, W>
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
        Self { program, runtime }
    }

    /// Run the program.
    pub fn run(mut self) -> Result<(), RuntimeError> {
        self.run_internal(self.program.instructions())
    }

    fn run_internal(&mut self, instructions: &[Instruction]) -> Result<(), RuntimeError> {
        for inst in instructions {
            while let NextAction::StepIn(sub) = self.runtime.exec_one(inst)? {
                self.run_internal(sub)?;
            }
        }

        Ok(())
    }
}
