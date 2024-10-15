pub use error::ParsingError;
pub use instruction::Instruction;

mod error;
mod instruction;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct Parser {
    contents: Vec<u8>,
    idx: usize,
    tokens: Vec<Instruction>,
    jump_stack: Vec<(usize, usize)>,
}

impl Parser {
    pub fn new(contents: Vec<u8>) -> Self {
        Self {
            contents,
            idx: 0,
            ..Default::default()
        }
    }

    pub fn parse(mut self) -> Result<Vec<Instruction>, ParsingError> {
        while let Some(byte) = self.next() {
            let token = match byte {
                b'+' => self.parse_add(1),
                b'-' => self.parse_add(1u8.wrapping_neg()),
                b'>' => self.parse_move(1),
                b'<' => self.parse_move(-1),
                b',' => Instruction::In,
                b'.' => Instruction::Out,
                b'[' => {
                    self.jump_stack.push((self.tokens.len(), self.idx - 1));

                    Instruction::JumpIfZero(0)
                }
                b']' => {
                    if let Some((idx, _)) = self.jump_stack.pop() {
                        if let Some(token) = self.try_parse_clear() {
                            token
                        } else if let Some(token) = self.try_parse_add_to() {
                            token
                        } else if let Some(token) = self.try_parse_move_until_0() {
                            token
                        } else {
                            self.tokens[idx] = Instruction::JumpIfZero(self.tokens.len());

                            Instruction::JumpIfNotZero(idx)
                        }
                    } else {
                        return Err(ParsingError::UnopenedBracket(self.idx - 1));
                    }
                }
                _ => continue,
            };

            self.tokens.push(token);
        }

        if let Some((_, idx)) = self.jump_stack.pop() {
            return Err(ParsingError::UnclosedBracket(idx));
        }

        Ok(self.tokens)
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

    fn try_parse_clear(&mut self) -> Option<Instruction> {
        use Instruction::*;

        match self.tokens.as_slice() {
            [.., JumpIfZero(_), Add(n)] if n % 2 == 1 => {
                self.remove_n(2);

                Some(Clear)
            }
            _ => None,
        }
    }

    fn try_parse_add_to(&mut self) -> Option<Instruction> {
        use Instruction::*;

        match self.tokens.as_slice() {
            // 255 is actually -1
            &[.., JumpIfZero(_), Add(255), Move(x), Add(1), Move(y)] if x.abs_diff(y) == 0 => {
                self.remove_n(5);

                Some(Instruction::AddTo(x))
            }
            _ => None,
        }
    }

    fn try_parse_move_until_0(&mut self) -> Option<Instruction> {
        use Instruction::*;

        match self.tokens.as_slice() {
            &[.., JumpIfZero(_), Move(n)] => {
                self.remove_n(2);

                Some(Instruction::MoveUntil(n))
            }
            _ => None,
        }
    }

    fn remove_n(&mut self, count: usize) {
        self.tokens.drain(self.tokens.len() - count..);
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
