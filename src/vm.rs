use std::collections::HashMap;

use chunk::Chunk;
use inst::{Instr, OpCode};
use ustr::Ustr;

use crate::{
    interpreter::{exception, Exception},
    value::Value,
};

pub mod chunk;
pub mod disassembler;
pub mod inst;

pub struct VM {
    stack: Vec<Value>,
    globals: HashMap<(Ustr, Ustr), Value>,
    chunk: Chunk,
    ip: usize,
}

impl VM {
    pub fn new(chunk: Chunk) -> Self {
        VM {
            stack: Vec::new(),
            globals: HashMap::new(),
            chunk,
            ip: 0,
        }
    }

    pub fn run(&mut self) -> Result<Value, Exception> {
        loop {
            let ip = self.ip;
            self.ip += 1;

            match self.chunk.code[ip] {
                Instr {
                    op: OpCode::Constant,
                    arg0,
                    ..
                } => {
                    self.stack.push(self.chunk.constants[arg0 as usize].clone());
                }
                Instr {
                    op: OpCode::GetGlobal,
                    arg0,
                    arg1,
                    ..
                } => {
                    let key = (self.read_symbol(arg0), self.read_symbol(arg1));
                    if let Some(value) = self.globals.get(&key) {
                        self.stack.push(value.clone());
                    } else {
                        exception!("Global not found: {:?}", key);
                    }
                }
                Instr {
                    op: OpCode::SetGlobal,
                    arg0,
                    arg1,
                    ..
                } => {
                    let key = (self.read_symbol(arg0), self.read_symbol(arg1));
                    let value = self.stack.pop().unwrap();
                    self.globals.insert(key, value);
                }
                Instr {
                    op: OpCode::Call,
                    arg0: _,
                    ..
                } => {
                    todo!()
                }
                Instr {
                    op: OpCode::Return, ..
                } => {
                    return Ok(self.stack.pop().unwrap());
                }
            }
        }
    }

    pub fn read_symbol(&self, index: u8) -> Ustr {
        match &self.chunk.constants[index as usize] {
            Value::Symbol(s) => *s,
            other => panic!("Expected symbol, got {}", other),
        }
    }
}
