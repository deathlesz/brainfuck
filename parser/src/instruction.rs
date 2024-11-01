use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Instruction {
    Add(u8),
    Move(isize),
    In,
    Out,
    JumpIfZero(usize),
    JumpIfNotZero(usize),

    Clear,
    Multiply(isize, u8),
    MoveUntilZero(isize),
}

impl Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Instruction::*;

        match *self {
            Add(count) => {
                let symbol = if (count as i8) < 0 { "-" } else { "+" };

                write!(
                    f,
                    "{}",
                    symbol.repeat((count as i8).unsigned_abs() as usize)
                )
            }
            Move(count) => {
                let symbol = if count < 0 { "<" } else { ">" };

                write!(f, "{}", symbol.repeat(count.unsigned_abs()))
            }
            In => write!(f, ","),
            Out => write!(f, "."),
            JumpIfZero(_) => write!(f, "["),
            JumpIfNotZero(_) => write!(f, "]"),
            Clear => write!(f, "[-]"),
            Multiply(offset, mult) => {
                let (symbol, symbol_opposite) = if offset < 0 { ("<", ">") } else { (">", "<") };
                let add = if (mult as i8) < 0 { "-" } else { "+" };

                write!(
                    f,
                    "[-{}{}{}]",
                    symbol.repeat(offset as usize),
                    add.repeat((mult as i8).unsigned_abs() as usize),
                    symbol_opposite.repeat(offset as usize),
                )
            }
            MoveUntilZero(count) => {
                let symbol = if count < 0 { "<" } else { ">" };

                write!(f, "[{}]", symbol.repeat(count.unsigned_abs()))
            }
        }
    }
}
