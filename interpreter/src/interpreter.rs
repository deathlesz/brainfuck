use color_eyre::{eyre::Context, Result};

use parser::Instruction;

#[derive(Debug, Clone)]
pub struct Interpreter<const N: usize> {
    memory: [u8; N],
    memptr: usize,
    instructions: Vec<Instruction>,
    instptr: usize,
}

impl<const N: usize> Interpreter<N> {
    const LENGTH: isize = N as isize;

    pub const fn new(instructions: Vec<Instruction>) -> Self {
        Self {
            memory: [0u8; N],
            memptr: 0,
            instructions,
            instptr: 0,
        }
    }

    pub fn run(mut self) -> Result<()> {
        let mut stdout = std::io::stdout().lock();
        let mut stdin = std::io::stdin().lock();

        while let Some(instruction) = self.instructions.get(self.instptr) {
            use Instruction::*;

            match instruction {
                Add(n) => self.memory[self.memptr] = self.memory[self.memptr].wrapping_add(*n),
                Move(n) => {
                    let n = (Self::LENGTH + n % Self::LENGTH) as usize;

                    self.memptr = (self.memptr + n) % Self::LENGTH as usize;
                }
                In => {
                    use std::io::Read;

                    stdin
                        .read_exact(&mut self.memory[self.memptr..self.memptr + 1])
                        .wrap_err("failed to read from stdin")?;
                }
                Out => {
                    use std::io::Write;

                    write!(stdout, "{}", self.memory[self.memptr] as char)
                        .wrap_err("failed to write to stdout")?;
                }
                JumpIfZero(to) if self.memory[self.memptr] == 0 => {
                    self.instptr = *to;
                }
                JumpIfNotZero(to) if self.memory[self.memptr] != 0 => {
                    self.instptr = *to;
                }
                Clear => self.memory[self.memptr] = 0,
                Multiply(offset, by) => {
                    let n = (Self::LENGTH + offset % Self::LENGTH) as usize;
                    let to = (self.memptr + n) % Self::LENGTH as usize;

                    let imm = self.memory[self.memptr].wrapping_mul(*by);
                    self.memory[to] = self.memory[to].wrapping_add(imm);
                    self.memory[self.memptr] = 0;
                }
                MoveUntilZero(n) => {
                    let n = (Self::LENGTH + n % Self::LENGTH) as usize;
                    while self.memory[self.memptr] != 0 {
                        self.memptr = (self.memptr + n) % Self::LENGTH as usize;
                    }
                }
                _ => {}
            }

            self.instptr += 1;
        }

        Ok(())
    }
}
