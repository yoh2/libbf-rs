use std::ops::Index;

/// Parsed program
#[derive(Debug)]
pub struct Program(Vec<Instruction>);

/// Brainf*ck instruction
#[derive(Debug, PartialEq, Eq)]
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

impl ProgramIndex {
    #[cfg(test)]
    pub fn new_for_test(index: impl Into<Vec<usize>>) -> Self {
        Self(index.into())
    }

    /// Set the index to point to the first instruction of the next depth.
    pub fn step_in(&mut self) {
        self.0.push(0);
    }

    /// Set the index to point to the previous depth.
    ///
    /// Returns `false` if the index pointed nothing.
    pub fn step_out(&mut self) -> bool {
        self.0.pop();
        !self.0.is_empty()
    }
}

impl Program {
    pub fn new(instructions: impl Into<Vec<Instruction>>) -> Self {
        Self(instructions.into())
    }

    pub fn instructions(&self) -> &[Instruction] {
        &self.0
    }

    pub fn first_index(&self) -> Option<ProgramIndex> {
        if self.0.is_empty() {
            None
        } else {
            Some(ProgramIndex(vec![0]))
        }
    }

    pub fn step_index(&self, index: &mut ProgramIndex) -> bool {
        Self::next_index_internal(self.instructions(), &mut index.0)
    }

    fn next_index_internal(instructions: &[Instruction], index: &mut [usize]) -> bool {
        let (head, tail) = index.split_first_mut().expect("index must not be empty");
        if tail.is_empty() {
            if *head + 1 < instructions.len() {
                *head += 1;
                true
            } else {
                false
            }
        } else if let Instruction::UntilZero(sub) = &instructions[*head] {
            Self::next_index_internal(sub, tail)
        } else {
            panic!("recursive index points to non-loop instruction")
        }
    }
}

impl Index<&ProgramIndex> for Program {
    type Output = Instruction;

    fn index(&self, index: &ProgramIndex) -> &Self::Output {
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
    use super::*;

    #[test]
    fn empty_first_index() {
        let program = Program::new([]);
        assert_eq!(program.first_index(), None);
    }

    #[test]
    fn first_index() {
        let program = Program::new([Instruction::PAdd(3)]);
        assert_eq!(
            program.first_index(),
            Some(ProgramIndex::new_for_test(vec![0]))
        );
    }

    #[test]
    fn index() {
        use Instruction::*;
        let program = Program::new([PAdd(1), UntilZero(vec![PAdd(2), Input, PAdd(-2)])]);

        assert_eq!(program[&ProgramIndex::new_for_test([0])], PAdd(1));
        assert_eq!(
            program[&ProgramIndex::new_for_test([1])],
            UntilZero(vec![PAdd(2), Input, PAdd(-2),])
        );
        assert_eq!(program[&ProgramIndex::new_for_test([1, 2])], PAdd(-2));
    }

    #[test]
    #[should_panic]
    fn empty_index() {
        let program = Program::new([Instruction::PAdd(3)]);
        let _ = program[&ProgramIndex::new_for_test([])];
    }

    #[test]
    #[should_panic]
    fn index_out_of_range() {
        use Instruction::*;
        let program = Program::new([PAdd(1), UntilZero(vec![PAdd(2), Input, PAdd(-2)])]);
        let _ = program[&ProgramIndex::new_for_test([2])];
    }

    #[test]
    #[should_panic]
    fn index_out_of_range2() {
        use Instruction::*;
        let program = Program::new([PAdd(1), UntilZero(vec![PAdd(2), Input, PAdd(-2)])]);
        let _ = program[&ProgramIndex::new_for_test([1, 3])];
    }

    #[test]
    #[should_panic]
    fn index_non_loop_recursion() {
        use Instruction::*;
        let program = Program::new([PAdd(1), UntilZero(vec![PAdd(2), Input, PAdd(-2)])]);
        let _ = program[&ProgramIndex::new_for_test([0, 0])];
    }
}
