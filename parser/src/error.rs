use std::fmt::Display;

#[derive(Debug, Clone)]
pub enum UnbalancedBrackets {
    UnclosedBracket(usize),
    UnopenedBracket(usize),
}

impl Display for UnbalancedBrackets {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use UnbalancedBrackets::*;

        match self {
            UnclosedBracket(pos) => {
                write!(f, "expected ']' to close '[' at position {}, got EOF", pos)
            }
            UnopenedBracket(pos) => {
                write!(f, "unexpected ']' at position {}", pos)
            }
        }
    }
}

impl std::error::Error for UnbalancedBrackets {}
