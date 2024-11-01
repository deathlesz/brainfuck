#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct OptimizationOptions {
    pub contract: bool,
    pub clear: bool,
    pub multiply: bool,
    pub move_until_zero: bool,
}

macro_rules! opt {
    ($name:ident, $with:ident) => {
        pub const fn $with(self) -> Self {
            Self {
                $name: true,
                ..self
            }
        }
    };
}

impl OptimizationOptions {
    pub const fn new() -> Self {
        Self {
            contract: false,
            clear: false,
            multiply: false,
            move_until_zero: false,
        }
    }

    pub const fn all() -> Self {
        Self {
            contract: true,
            clear: true,
            multiply: true,
            move_until_zero: true,
        }
    }

    opt!(contract, with_contract);
    opt!(clear, with_clear);
    opt!(multiply, with_multiply);
    opt!(move_until_zero, with_move_until_zero);
}
