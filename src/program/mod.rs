/// Parsed program
pub type Program = Vec<Instruction>;

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
    UntilZero(Program),
}
