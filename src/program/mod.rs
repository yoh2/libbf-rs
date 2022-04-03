use std::ops::Index;

/// Parsed program
pub struct Program(Vec<Instruction>);

/// Brainf*ck instruction
#[derive(Debug)]
pub enum Instruction {
    /// Unified pointer increments/decrements
    PAdd(isize),

    /// Unified data increesments/decrements
    DAdd(isize),

    /// Write one byte at the current pointer
    Output,

    /// Read one byte  and store it at the current pointer
    Input,

    /// loop until the value at the current pointer is non-zero
    UntilZero(Vec<Instruction>),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ProgramIndex(Vec<usize>);

impl Program {
    pub fn new(instructions: impl Into<Vec<Instruction>>) -> Self {
        Self(instructions.into())
    }

    pub fn instructions(&self) -> &[Instruction] {
        &self.0
    }
}

impl Index<ProgramIndex> for Program {
    type Output = Instruction;

    fn index(&self, index: ProgramIndex) -> &Self::Output {
        instruction_at(&self.0, &index.0)
    }
}

fn instruction_at<'a>(instructions: &'a [Instruction], index: &[usize]) -> &'a Instruction {
    assert!(!index.is_empty());
    let (head, tail) = index.split_first().expect("index must not be empty");
    let instruction = &instructions[*head];
    if tail.is_empty() {
        instruction
    } else if let Instruction::UntilZero(sub) = instruction {
        instruction_at(sub, tail)
    } else {
        panic!("recursive index points to non-loop instruction")
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn index_test() {}
}
