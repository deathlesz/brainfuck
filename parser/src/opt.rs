#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct OptimizationOptions {
    pub contract: bool,
    pub clear: bool,
    pub add_to: bool,
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
            add_to: false,
            move_until_zero: false,
        }
    }

    pub const fn all() -> Self {
        Self {
            contract: true,
            clear: true,
            add_to: true,
            move_until_zero: true,
        }
    }

    opt!(contract, with_contract);
    opt!(clear, with_clear);
    opt!(add_to, with_add_to);
    opt!(move_until_zero, with_move_until_zero);
}
