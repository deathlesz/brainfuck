use color_eyre::{eyre::Context as _, Result};

use dynasmrt::{dynasm, mmap::MutableBuffer, DynasmApi as _, DynasmLabelApi as _};
use parser::Instruction;

#[derive(Debug, Clone)]
pub struct Compiler<const N: i32> {
    instructions: Vec<Instruction>,
}

impl<const N: i32> Compiler<N> {
    pub fn new(instructions: Vec<Instruction>) -> Self {
        Self { instructions }
    }

    pub fn run(self) -> Result<()> {
        let mut ops = dynasmrt::x64::Assembler::new().wrap_err("failed to allocate memory")?;

        // r12 will be the address of `memory`
        // r13 will be the value of `pointer`
        // r12 is got from argument 1 in `rdi`
        // r13 is set to 0
        dynasm! { ops
            ; .arch x64
            ; push rbp
            ; mov rbp, rsp
            ; push r12
            ; push r13
            ; mov r12, rdi
            ; xor r13, r13
        };

        use Instruction::*;

        let mut bracket_stack = Vec::new();
        for instruction in self.instructions {
            match instruction {
                Add(n) => dynasm! { ops
                    ; .arch x64
                    ; add BYTE [r12 + r13], BYTE n as i8
                },
                Move(n) => {
                    let n = n as i32;

                    if n > 0 {
                        dynasm! { ops
                            ; .arch x64
                            ; lea eax, [r13 + n]
                            ; add r13, (-N - n)
                            ; cmp eax, N
                            ; cmovl r13d, eax
                        }
                    } else {
                        dynasm! { ops
                            ; .arch x64
                            ; lea eax, [r13 + n]
                            ; add r13d, N + n
                            ; test eax, eax
                            ; cmovns r13d, eax
                        }
                    }
                }
                In => dynasm! { ops
                    ; .arch x64
                    ; mov rax, QWORD Self::read as *const () as i64
                    ; lea rdi, [r12 + r13]
                    ; call rax
                    ; test rax,rax
                    ; jne ->exit
                },
                Out => dynasm! { ops
                    ; .arch x64
                    ; mov rax, QWORD Self::write as *const () as i64
                    ; mov rdi, [r12 + r13]
                    ; call rax
                    ; test rax,rax
                    ; jne ->exit
                },
                JumpIfZero(_) => {
                    let start_label = ops.new_dynamic_label();
                    let end_label = ops.new_dynamic_label();

                    dynasm! { ops
                        ; .arch x64
                        ; cmp BYTE [r12+r13], 0
                        ; je =>end_label
                        ; =>start_label
                    };

                    bracket_stack.push((start_label, end_label));
                }
                JumpIfNotZero(_) => {
                    let (start_label, end_label) = bracket_stack.pop().unwrap(); // will never fail
                    dynasm! { ops
                        ; .arch x64
                        ; cmp BYTE [r12+r13], 0
                        ; jne =>start_label
                        ; =>end_label
                    };
                }
                Clear => dynasm! { ops
                    ; .arch x64
                    ; mov BYTE [r12+r13], 0
                },
                AddTo(n) => {
                    let n = n as i32;

                    dynasm! { ops
                        ; .arch x64
                        ;;
                        if n > 0 {
                            dynasm! { ops
                                ; lea ecx, [r13 + n]
                                ; lea eax, [r13 + n - N]
                                ; cmp ecx, N
                                ; cmovl eax, ecx
                            }
                        } else {
                            dynasm! { ops
                                ; lea ecx, [r13 + n]
                                ; lea eax, [r13 + N + n]
                                ; test ecx, ecx
                                ; cmovns eax, ecx
                            }
                        }
                        ; mov cl, [r12 + r13]
                        ; add BYTE [r12 + rax], cl
                        ; mov BYTE [r12 + r13], 0
                    }
                }
                MoveUntilZero(n) => {
                    let n = n as i32;

                    dynasm! { ops
                        ; .arch x64
                        ; repeat:
                        ; cmp BYTE [r12 + r13], 0
                        ; je >exit
                        ;;
                        if n > 0 {
                            dynasm! { ops
                                ; .arch x64
                                ; lea eax, [r13 + n]
                                ; add r13, (-N - n)
                                ; cmp eax, N
                                ; cmovl r13d, eax
                            }
                        } else {
                            dynasm! { ops
                                ; .arch x64
                                ; lea eax, [r13 + n]
                                ; add r13d, N + n
                                ; test eax, eax
                                ; cmovns r13d, eax
                            }
                        }
                        ; jmp <repeat
                        ; exit:
                    }
                }
            }
        }

        dynasm! { ops
            ; .arch x64
            ; xor rax, rax
            ; ->exit:
            ; pop r13
            ; pop r12
            ; pop rbp
            ; ret
        };

        let code = ops.finalize().unwrap(); // should never fail
        let mut buffer = MutableBuffer::new(code.len()).wrap_err("failed to allocate memory")?;
        buffer.set_len(code.len());

        buffer.copy_from_slice(&code);

        let buffer = buffer
            .make_exec()
            .wrap_err("failed to make memory executable")?;
        let mut memory = [0u8; 30_000];
        unsafe {
            let code_fn: unsafe extern "sysv64" fn(*mut u8) -> *mut std::io::Error =
                std::mem::transmute(buffer.as_ptr());

            let error = code_fn(memory.as_mut_ptr());

            if !error.is_null() {
                return Err(*Box::from_raw(error))?;
            }
        }

        Ok(())
    }

    extern "sysv64" fn write(value: u8) -> *mut std::io::Error {
        use std::io::Write;

        let mut stdout = std::io::stdout().lock();
        let result = stdout.write_all(&[value]);

        match result {
            Err(err) => Box::into_raw(Box::new(err)),
            _ => std::ptr::null_mut(),
        }
    }

    extern "sysv64" fn read(buf: *mut u8) -> *mut std::io::Error {
        use std::io::Read;

        let mut stdin = std::io::stdin().lock();
        let result = stdin.read_exact(unsafe { std::slice::from_raw_parts_mut(buf, 1) });

        match result {
            Err(err) => Box::into_raw(Box::new(err)),
            _ => std::ptr::null_mut(),
        }
    }
}
