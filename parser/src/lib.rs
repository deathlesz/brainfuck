pub use error::UnbalancedBrackets;
pub use instruction::Instruction;

mod error;
mod instruction;

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

    pub fn parse(mut self) -> Result<Vec<Instruction>, UnbalancedBrackets> {
        while let Some(byte) = self.next() {
            let instruction = match byte {
                b'+' => self.parse_add(1),
                b'-' => self.parse_add(1u8.wrapping_neg()),
                b'>' => self.parse_move(1),
                b'<' => self.parse_move(-1),
                b',' => Instruction::In,
                b'.' => Instruction::Out,
                b'[' => {
                    self.jump_stack
                        .push((self.instructions.len(), self.idx - 1));

                    Instruction::JumpIfZero(0)
                }
                b']' => {
                    if let Some((idx, _)) = self.jump_stack.pop() {
                        if let Some(clear) = self.try_parse_clear() {
                            clear
                        } else if let Some(add_to) = self.try_parse_add_to() {
                            add_to
                        } else if let Some(move_until) = self.try_parse_move_until_zero() {
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

    #[cfg(feature = "optimize_clear")]
    fn try_parse_clear(&mut self) -> Option<Instruction> {
        use Instruction::*;

        match self.instructions.as_slice() {
            [.., JumpIfZero(_), Add(n)] if n % 2 == 1 => {
                self.remove_n(2);

                Some(Clear)
            }
            _ => None,
        }
    }

    #[cfg(not(feature = "optimize_clear"))]
    fn try_parse_clear(&mut self) -> Option<Instruction> {
        None
    }

    #[cfg(feature = "optimize_add_to")]
    fn try_parse_add_to(&mut self) -> Option<Instruction> {
        use Instruction::*;

        match self.instructions.as_slice() {
            // 255 is actually -1
            &[.., JumpIfZero(_), Add(255), Move(x), Add(1), Move(y)] if x - y.abs() == 0 => {
                self.remove_n(5);

                Some(Instruction::AddTo(x))
            }
            _ => None,
        }
    }

    #[cfg(not(feature = "optimize_add_to"))]
    fn try_parse_add_to(&mut self) -> Option<Instruction> {
        None
    }

    #[cfg(feature = "optimize_move_until_zero")]
    fn try_parse_move_until_zero(&mut self) -> Option<Instruction> {
        use Instruction::*;

        match self.instructions.as_slice() {
            &[.., JumpIfZero(_), Move(n)] => {
                self.remove_n(2);

                Some(Instruction::MoveUntilZero(n))
            }
            _ => None,
        }
    }

    #[cfg(not(feature = "optimize_move_until_zero"))]
    fn try_parse_move_until_zero(&mut self) -> Option<Instruction> {
        None
    }

    #[cfg(any(
        feature = "optimize_clear",
        feature = "optimize_add_to",
        feature = "optimize_move_until_zero"
    ))]
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
