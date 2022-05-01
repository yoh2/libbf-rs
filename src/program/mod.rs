//! Parsed program of Brainfuck-like language and related definitions.
use crate::token::TokenInfo;
use std::ops::Index;

/// A parsed program with token informations.
#[derive(Debug)]
pub struct FatProgram<'a>(Vec<FatInstruction<'a>>);

#[derive(Debug, PartialEq, Eq)]
pub struct FatInstruction<'a> {
    pub kind: FatInstructionKind<'a>,
    pub tokens: Vec<TokenInfo<'a>>,
}

/// An intermediate instruction with corresponding token information of Brainfuck-like language.
#[derive(Debug, PartialEq, Eq)]
pub enum FatInstructionKind<'a> {
    /// Unified pointer increments/decrements
    PAdd(isize),

    /// Unified data increesments/decrements
    DAdd(isize),

    /// Write one byte at the current pointer
    Output,

    /// Read one byte and store it at the current pointer
    Input,

    /// Loop until the value at the current pointer is non-zero
    UntilZero(Vec<FatInstruction<'a>>),

    /// No-operation, which corresponds to a sequence of PIncs and PDecs of the same number or
    /// DIncs and DDecs of the same number.
    Nop,
}

impl<'a> FatInstruction<'a> {
    pub fn as_instruction(&self) -> Option<Instruction> {
        self.kind.as_instruction()
    }
}

impl<'a> FatInstructionKind<'a> {
    pub fn as_instruction(&self) -> Option<Instruction> {
        match self {
            FatInstructionKind::PAdd(n) => Some(Instruction::PAdd(*n)),
            FatInstructionKind::DAdd(n) => Some(Instruction::DAdd(*n)),
            FatInstructionKind::Output => Some(Instruction::Output),
            FatInstructionKind::Input => Some(Instruction::Input),
            FatInstructionKind::UntilZero(insts) => Some(Instruction::UntilZero(
                fat_instructions_to_instructins(insts),
            )),
            FatInstructionKind::Nop => None,
        }
    }
}

impl<'a> FatProgram<'a> {
    pub fn new(instructions: Vec<FatInstruction<'a>>) -> Self {
        FatProgram(instructions)
    }

    pub fn instructions(&self) -> &[FatInstruction<'a>] {
        &self.0
    }

    pub fn as_program(&self) -> Program {
        Program::new(fat_instructions_to_instructins(self.instructions()))
    }
}

impl<'a> From<FatProgram<'a>> for Program {
    fn from(x: FatProgram<'a>) -> Self {
        x.as_program()
    }
}

fn fat_instructions_to_instructins(fat_instructions: &[FatInstruction]) -> Vec<Instruction> {
    fat_instructions
        .iter()
        .filter_map(|inst| inst.kind.as_instruction())
        .collect()
}

/// A parsed program of Brainfuck-link language.
///
/// Each instruction can be acceseed by [`ProgramIndex`].
#[derive(Debug)]
pub struct Program(Vec<Instruction>);

/// An intermediate instruction of Brainfuck-like language.
#[derive(Debug, PartialEq, Eq)]
pub enum Instruction {
    /// Unified pointer increments/decrements
    PAdd(isize),

    /// Unified data increesments/decrements
    DAdd(isize),

    /// Write one byte at the current pointer
    Output,

    /// Read one byte and store it at the current pointer
    Input,

    /// loop until the value at the current pointer is non-zero
    UntilZero(Vec<Instruction>),
}

/// An itdex for [`Program`]
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
    /// Create a new program from an [`Instruction`] vector.
    pub fn new(instructions: impl Into<Vec<Instruction>>) -> Self {
        Self(instructions.into())
    }

    /// Get the instructions of the program.
    pub fn instructions(&self) -> &[Instruction] {
        &self.0
    }

    /// Get an indef which points the first instruction of the program.
    ///
    /// If instructins are empty, returns `None`.
    pub fn first_index(&self) -> Option<ProgramIndex> {
        if self.0.is_empty() {
            None
        } else {
            Some(ProgramIndex(vec![0]))
        }
    }

    /// Step `index` to the next instruction.
    ///
    /// If the index already points to the last instruction of the program or
    /// the last instruction of the sub-instructions (in [`Instruction::UntilZero`] instruction),
    /// the index is not changed and this function returns `false`.
    /// Otherwise, the index is changed to point to the next instruction and
    /// this function returns `true`.
    ///
    /// This function does not step into sub-instructions of [`Instruction::UntilZero`] instruction.
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
    fn fat_program_as_program() {
        let fat_program = FatProgram::new(vec![
            FatInstruction {
                kind: FatInstructionKind::PAdd(1),
                tokens: vec![],
            },
            FatInstruction {
                kind: FatInstructionKind::DAdd(2),
                tokens: vec![],
            },
            FatInstruction {
                kind: FatInstructionKind::Input,
                tokens: vec![],
            },
            FatInstruction {
                kind: FatInstructionKind::Output,
                tokens: vec![],
            },
            FatInstruction {
                kind: FatInstructionKind::UntilZero(vec![
                    FatInstruction {
                        kind: FatInstructionKind::PAdd(-1),
                        tokens: vec![],
                    },
                    FatInstruction {
                        kind: FatInstructionKind::DAdd(-2),
                        tokens: vec![],
                    },
                ]),
                tokens: vec![],
            },
        ]);

        let expected = [
            Instruction::PAdd(1),
            Instruction::DAdd(2),
            Instruction::Input,
            Instruction::Output,
            Instruction::UntilZero(vec![Instruction::PAdd(-1), Instruction::DAdd(-2)]),
        ];

        assert_eq!(fat_program.as_program().instructions(), &expected);
        assert_eq!(Program::from(fat_program).instructions(), &expected);
    }

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
