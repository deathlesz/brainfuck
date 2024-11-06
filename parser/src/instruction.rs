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
                    symbol.repeat(offset.unsigned_abs()),
                    add.repeat((mult as i8).unsigned_abs() as usize),
                    symbol_opposite.repeat(offset.unsigned_abs()),
                )
            }
            MoveUntilZero(count) => {
                let symbol = if count < 0 { "<" } else { ">" };

                write!(f, "[{}]", symbol.repeat(count.unsigned_abs()))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Instruction::*;

    macro_rules! test {
        ($name:ident,$inst:expr,$result:expr) => {
            #[test]
            fn $name() {
                assert_eq!($inst.to_string(), $result)
            }
        };
    }

    test!(display_add_positive, Add(10), "++++++++++");
    test!(display_add_negative, Add(5u8.wrapping_neg()), "-----");
    test!(display_move_positive, Move(12), ">>>>>>>>>>>>");
    test!(display_move_negative, Move(-7), "<<<<<<<");
    test!(display_in, In, ",");
    test!(display_out, Out, ".");
    test!(display_jz, JumpIfZero(81), "[");
    test!(display_jne, JumpIfNotZero(1), "]");
    test!(display_clear, Clear, "[-]");
    test!(
        display_multiply_positive_positive,
        Multiply(5, 5),
        "[->>>>>+++++<<<<<]"
    );
    test!(
        display_multiply_positive_negative,
        Multiply(2, 3u8.wrapping_neg()),
        "[->>---<<]"
    );
    test!(
        display_multiply_negative_positive,
        Multiply(-7, 2),
        "[-<<<<<<<++>>>>>>>]"
    );
    test!(
        display_multiply_negative_negative,
        Multiply(-1, 4u8.wrapping_neg()),
        "[-<---->]"
    );
    test!(display_move_until_zero_positive, MoveUntilZero(3), "[>>>]");
    test!(
        display_move_until_zero_negative,
        MoveUntilZero(-9),
        "[<<<<<<<<<]"
    );
}
