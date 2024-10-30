use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Instruction {
    Add(u8),
    Move(isize),
    In,
    Out,
    JumpIfZero(usize),
    JumpIfNotZero(usize),

    #[cfg(feature = "optimize_clear")]
    Clear,
    #[cfg(feature = "optimize_add_to")]
    AddTo(isize),
    #[cfg(feature = "optimize_move_until_zero")]
    MoveUntilZero(isize),
}

impl Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Instruction::*;

        let symbol = match *self {
            Add(count) => {
                let symbol = if (count as i8) < 0 { "-" } else { "+" };
                symbol.repeat(count as usize)
            }
            Move(count) => {
                let symbol = if count < 0 { "<" } else { ">" };

                symbol.repeat(count as usize)
            }
            In => ".".into(),
            Out => ",".into(),
            JumpIfZero(_) => "[".into(),
            JumpIfNotZero(_) => "]".into(),
            #[cfg(feature = "optimize_clear")]
            Clear => "[-]".into(),
            #[cfg(feature = "optimize_add_to")]
            AddTo(offset) => {
                let (symbol, symbol_opposite) = if offset < 0 { ("<", ">") } else { (">", "<") };

                let moves = symbol.repeat(offset as usize);
                let moves_opposite = symbol_opposite.repeat(offset as usize);

                format!("[-{moves}+{moves_opposite}]")
            }
            #[cfg(feature = "optimize_move_until_zero")]
            MoveUntilZero(count) => {
                let symbol = if count < 0 { "<" } else { ">" };

                format!("[{}]", symbol.repeat(count as usize))
            }
        };

        write!(f, "{symbol}")
    }
}
