#[macro_use]
pub mod error;
pub mod memory;
pub mod opcode;
pub mod stack;

use error::VmError;
use opcode::{Control, Opcode};
use std::fmt;

pub struct Vm {
    stack: stack::Stack,
    memory: memory::Memory,
    pc: usize, // program counter
    code: Vec<u8>,
    valid_jumps: Vec<usize>,
    return_data: (usize, usize),
}

#[derive(Debug)]
pub enum Execution {
    Continue,
    Stop,
    Revert,
    Return,
    Error(VmError),
}

impl Vm {
    pub fn new(code: &[u8]) -> Self {
        // determine valid jumps
        let valid_jumps = {
            let mut jumps = Vec::new();
            let mut i = 0;
            while i < code.len() {
                let inc = if code[i] == Opcode::JUMPDEST.code {
                    jumps.push(i);
                    1
                } else if code[i] >= Opcode::PUSH1.code && code[i] <= Opcode::PUSH32.code {
                    (code[i] - Opcode::PUSH1.code + 1) as usize
                } else {
                    1
                };

                i += inc;
            }
            jumps
        };

        Vm {
            stack: stack::Stack::default(),
            memory: memory::Memory::new(),
            pc: 0,
            code: code.to_vec(),
            valid_jumps,
            return_data: (0, 0), // (offset, size)
        }
    }

    pub fn run(&mut self) -> Result<(), Execution> {
        loop {
            match self.step() {
                Ok(reason) => match reason {
                    Execution::Stop => {
                        info!("STOP");
                        return Ok(());
                    }
                    Execution::Return => {
                        info!("RETURN");
                        return Ok(());
                    }
                    _ => (()),
                },
                Err(e) => {
                    return Err(e);
                }
            }
        }
    }

    pub fn step(&mut self) -> Result<Execution, Execution> {
        if self.pc >= self.code.len() {
            return Ok(Execution::Stop);
        }

        let opcode = if let Some(op) = self.code.get(self.pc).and_then(|&code| Opcode::get(code)) {
            op
        } else {
            return Err(Execution::Error(VmError::InvalidOpcode));
        };

        let func = opcode.exec;

        info!("\n{:?}", self);
        match func(self) {
            Control::Continue(n) => {
                self.pc += n;
                Ok(Execution::Continue)
            }
            Control::Jump(dest) => {
                self.pc = dest;
                Ok(Execution::Continue)
            }
            Control::Return => Ok(Execution::Return),
            Control::Stop => Ok(Execution::Stop),
            Control::Revert => Err(Execution::Revert),
            Control::Error(e) => Err(Execution::Error(e)),
        }
    }

    pub fn get_return_data(&mut self) -> Vec<u8> {
        self.memory.read(self.return_data.0, self.return_data.1)
    }

    pub fn is_valid_jump(&self, dest: usize) -> bool {
        self.valid_jumps.contains(&dest)
    }
}

impl fmt::Debug for Vm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let op_name = if let Some(op) = self.code.get(self.pc).and_then(|&code| Opcode::get(code)) {
            op.mnemonic
        } else {
            Opcode::INVALID.mnemonic
        };
        write!(
            f,
            "pc: {:?} -> opcode: {}\nstack:\n{}\nmemory: {:?}",
            self.pc, op_name, self.stack, self.memory
        )
    }
}

#[cfg(test)]
mod test {
    use super::Vm;

    #[test]
    fn run() {
        let code = vec![0x60, 0x05, 0x00];
        let mut vm = Vm::new(&code);
        assert_eq!(vm.code.len(), 3);
        assert_eq!(vm.pc, 0);
        vm.step().ok();
        // assert_eq!(vm.pc, 1);
        // vm.run();
    }
}