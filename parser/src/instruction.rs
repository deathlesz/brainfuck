#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Instruction {
    Add(u8),
    Move(isize),
    In,
    Out,
    JumpIfZero(usize),
    JumpIfNotZero(usize),

    Clear,
    AddTo(isize),
    MoveUntil(isize),
}
