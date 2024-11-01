pub use error::UnbalancedBrackets;
pub use instruction::Instruction;
pub use opt::OptimizationOptions;

mod error;
mod instruction;
mod opt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Parser<'a> {
    contents: &'a [u8],
    idx: usize,
    instructions: Vec<Instruction>,
    jump_stack: Vec<(usize, usize)>,
}

impl<'a> Parser<'a> {
    pub fn new(contents: &'a [u8]) -> Self {
        Self {
            contents,
            idx: 0,
            instructions: Vec::new(),
            jump_stack: Vec::new(),
        }
    }

    pub fn parse(
        mut self,
        opts: OptimizationOptions,
    ) -> Result<Vec<Instruction>, UnbalancedBrackets> {
        while let Some(byte) = self.next() {
            let instruction = match byte {
                b'+' if opts.contract => self.parse_add(1),
                b'+' => Instruction::Add(1),
                b'-' if opts.contract => self.parse_add(1u8.wrapping_neg()),
                b'-' => Instruction::Add(1u8.wrapping_neg()),

                b'>' if opts.contract => self.parse_move(1),
                b'>' => Instruction::Move(1),
                b'<' if opts.contract => self.parse_move(-1),
                b'<' => Instruction::Move(-1),

                b',' => Instruction::In,
                b'.' => Instruction::Out,

                b'[' => {
                    self.jump_stack
                        .push((self.instructions.len(), self.idx - 1));

                    Instruction::JumpIfZero(0)
                }
                b']' => {
                    if let Some((idx, _)) = self.jump_stack.pop() {
                        if let Some(clear) = self.try_parse_clear(opts.clear) {
                            clear
                        } else if let Some(multiply) = self.try_parse_multiply(opts.multiply) {
                            multiply
                        } else if let Some(move_until) =
                            self.try_parse_move_until_zero(opts.move_until_zero)
                        {
                            move_until
                        } else {
                            self.instructions[idx] =
                                Instruction::JumpIfZero(self.instructions.len());

                            Instruction::JumpIfNotZero(idx)
                        }
                    } else {
                        return Err(UnbalancedBrackets::UnopenedBracket(self.idx - 1));
                    }
                }
                _ => continue,
            };

            match instruction {
                Instruction::Add(0) | Instruction::Move(0) => {}
                _ => self.instructions.push(instruction),
            }
        }

        if let Some((_, idx)) = self.jump_stack.pop() {
            return Err(UnbalancedBrackets::UnclosedBracket(idx));
        }

        Ok(self.instructions)
    }

    fn parse_add(&mut self, mut acc: u8) -> Instruction {
        while let Some(byte) = self.peek() {
            acc = match byte {
                b'+' => acc.wrapping_add(1),
                b'-' => acc.wrapping_sub(1),
                b'>' | b'<' | b'.' | b',' | b'[' | b']' => break,
                _ => acc,
            };

            self.idx += 1;
        }

        Instruction::Add(acc)
    }

    fn parse_move(&mut self, mut acc: isize) -> Instruction {
        while let Some(byte) = self.peek() {
            match byte {
                b'>' => acc += 1,
                b'<' => acc -= 1,
                b'+' | b'-' | b'.' | b',' | b'[' | b']' => break,
                _ => {}
            }

            self.idx += 1;
        }

        Instruction::Move(acc)
    }

    fn try_parse_clear(&mut self, opt: bool) -> Option<Instruction> {
        use Instruction::*;

        if !opt {
            return None;
        }

        match self.instructions.as_slice() {
            [.., JumpIfZero(_), Add(n)] if n % 2 == 1 => {
                self.remove_n(2);

                Some(Clear)
            }
            _ => None,
        }
    }

    fn try_parse_multiply(&mut self, opt: bool) -> Option<Instruction> {
        use Instruction::*;

        if !opt {
            return None;
        }

        match self.instructions.as_slice() {
            // 255 is actually -1
            &[.., JumpIfZero(_), Add(255), Move(x), Add(n), Move(y)] if x.abs() - y.abs() == 0 => {
                self.remove_n(5);

                Some(Instruction::Multiply(x, n))
            }
            _ => None,
        }
    }

    fn try_parse_move_until_zero(&mut self, opt: bool) -> Option<Instruction> {
        use Instruction::*;

        if !opt {
            return None;
        }

        match self.instructions.as_slice() {
            &[.., JumpIfZero(_), Move(n)] => {
                self.remove_n(2);

                Some(Instruction::MoveUntilZero(n))
            }
            _ => None,
        }
    }

    fn remove_n(&mut self, count: usize) {
        self.instructions.drain(self.instructions.len() - count..);
    }

    fn next(&mut self) -> Option<u8> {
        let byte = self.peek();
        self.idx += 1;

        byte
    }

    fn peek(&mut self) -> Option<u8> {
        self.contents.get(self.idx).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::{Instruction::*, *};

    macro_rules! test {
        ($(#[$attr:meta])? $name:ident($opts:expr), $input:expr => $output:expr) => {
            #[test]
            $(#[$attr])?
            fn $name() {
                let parser = Parser::new($input);
                let result = parser.parse($opts).expect("failed to parse");

                assert_eq!(&result, $output)
            }
        };
    }

    test!(parse_empty(OptimizationOptions::new()), b"" => &[]);
    test!(
        parse_add(OptimizationOptions::new().with_contract()),
        b"+++-+--++++----+++----+++--" =>
        &[Add(1)]
    );
    test!(parse_zero_add(OptimizationOptions::new().with_contract()), b"+++--+++--+-++--+---" => &[]);
    test!(
        parse_move(OptimizationOptions::new().with_contract()),
        b"<<<>>><><<><>>>><><><<<><><><>>><<<>><><>>>>>><>><<<<>" =>
        &[Move(4)]
    );
    test!(parse_zero_move(OptimizationOptions::new().with_contract()), b">>><<>>><<><>><<><<<" => &[]);
    test!(parse_in(OptimizationOptions::new()), b",,,,,,,,,," => &[Instruction::In].repeat(10));
    test!(parse_out(OptimizationOptions::new()), b".........." => &[Instruction::Out].repeat(10));
    test!(
        parse_jz_jnz(OptimizationOptions::new()),
        b"[[[[][]][[[]]]][[]]]" =>
        &[JumpIfZero(19), JumpIfZero(14), JumpIfZero(7), JumpIfZero(4), JumpIfNotZero(3), JumpIfZero(6), JumpIfNotZero(5), JumpIfNotZero(2), JumpIfZero(13), JumpIfZero(12), JumpIfZero(11), JumpIfNotZero(10), JumpIfNotZero(9), JumpIfNotZero(8), JumpIfNotZero(1), JumpIfZero(18), JumpIfZero(17), JumpIfNotZero(16), JumpIfNotZero(15), JumpIfNotZero(0)]
    );
    test!(
        #[should_panic] parse_fail_jz_unbalanced(OptimizationOptions::new()),
        b"[[+-++>><><><[++[[<<[>>>>[[+><><>>>]]<><>]" =>
        &[]
    );
    test!(
        #[should_panic] parse_fail_jnz_unbalanced(OptimizationOptions::new()),
        b"[+++[<.,>>++<<<<>++--+]]]" =>
        &[]
    );
    test!(
        parse_clear(OptimizationOptions::new().with_contract().with_clear()),
        b"[-][+++][--][+>+++-]" =>
        &[Clear, Clear, JumpIfZero(4), Add(2u8.wrapping_neg()), JumpIfNotZero(2), JumpIfZero(9), Add(1), Move(1), Add(2), JumpIfNotZero(5)]
    );
    test!(
        parse_multiply_1(OptimizationOptions::new().with_contract().with_multiply()),
        b"[->>>+<<<]" =>
        &[Multiply(3, 1)]
    );
    test!(
        parse_multiply_1_opposite(OptimizationOptions::new().with_contract().with_multiply()),
        b"[-<<<+>>>]" =>
        &[Multiply(-3, 1)]
    );
    test!(
        parse_move_until_zero(OptimizationOptions::new().with_contract().with_move_until_zero()),
        b"[>>>][>][><><>>>>><>][>>>+<[>]]" =>
        &[MoveUntilZero(3), MoveUntilZero(1), MoveUntilZero(5), JumpIfZero(8), Move(3), Add(1), Move(-1), MoveUntilZero(1), JumpIfNotZero(3)]
    );
}
