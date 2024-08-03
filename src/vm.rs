use std::collections::HashMap;

use chunk::Chunk;
use inst::{Instr, OpCode};
use ustr::Ustr;

use crate::{
    ast::{BinOp, UnaryOp},
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
                    arg0,
                    ..
                } => {
                    let args = self.stack.split_off(self.stack.len() - arg0 as usize);
                    let callee = self.stack.pop().unwrap();
                    let res = match callee {
                        Value::NativeFunc(ptr) => ptr(&args)?,
                        _ => todo!(),
                    };
                    self.stack.push(res);
                }

                Instr {
                    op: OpCode::Return, ..
                } => {
                    return Ok(self.stack.pop().unwrap());
                }
                Instr {
                    op: OpCode::Add, ..
                } => {
                    let rhs = self.stack.pop().unwrap();
                    let lhs = self.stack.pop().unwrap();
                    let res = Value::bin_op(BinOp::Add, &lhs, &rhs)?;
                    self.stack.push(res);
                }

                Instr {
                    op: OpCode::Sub, ..
                } => {
                    let rhs = self.stack.pop().unwrap();
                    let lhs = self.stack.pop().unwrap();
                    let res = Value::bin_op(BinOp::Sub, &lhs, &rhs)?;
                    self.stack.push(res);
                }

                Instr {
                    op: OpCode::Mul, ..
                } => {
                    let rhs = self.stack.pop().unwrap();
                    let lhs = self.stack.pop().unwrap();
                    let res = Value::bin_op(BinOp::Mul, &lhs, &rhs)?;
                    self.stack.push(res);
                }

                Instr {
                    op: OpCode::Div, ..
                } => {
                    let rhs = self.stack.pop().unwrap();
                    let lhs = self.stack.pop().unwrap();
                    let res = Value::bin_op(BinOp::Div, &lhs, &rhs)?;
                    self.stack.push(res);
                }

                Instr { op: OpCode::Eq, .. } => {
                    let rhs = self.stack.pop().unwrap();
                    let lhs = self.stack.pop().unwrap();
                    let res = Value::bin_op(BinOp::Eq, &lhs, &rhs)?;
                    self.stack.push(res);
                }

                Instr {
                    op: OpCode::Neq, ..
                } => {
                    let rhs = self.stack.pop().unwrap();
                    let lhs = self.stack.pop().unwrap();
                    let res = Value::bin_op(BinOp::Neq, &lhs, &rhs)?;
                    self.stack.push(res);
                }

                Instr { op: OpCode::Gt, .. } => {
                    let rhs = self.stack.pop().unwrap();
                    let lhs = self.stack.pop().unwrap();
                    let res = Value::bin_op(BinOp::Gt, &lhs, &rhs)?;
                    self.stack.push(res);
                }

                Instr {
                    op: OpCode::Gte, ..
                } => {
                    let rhs = self.stack.pop().unwrap();
                    let lhs = self.stack.pop().unwrap();
                    let res = Value::bin_op(BinOp::Gte, &lhs, &rhs)?;
                    self.stack.push(res);
                }

                Instr { op: OpCode::Lt, .. } => {
                    let rhs = self.stack.pop().unwrap();
                    let lhs = self.stack.pop().unwrap();
                    let res = Value::bin_op(BinOp::Lt, &lhs, &rhs)?;
                    self.stack.push(res);
                }

                Instr {
                    op: OpCode::Lte, ..
                } => {
                    let rhs = self.stack.pop().unwrap();
                    let lhs = self.stack.pop().unwrap();
                    let res = Value::bin_op(BinOp::Lte, &lhs, &rhs)?;
                    self.stack.push(res);
                }

                Instr {
                    op: OpCode::Neg, ..
                } => {
                    let rhs = self.stack.pop().unwrap();
                    let res = Value::unary_op(UnaryOp::Neg, &rhs)?;
                    self.stack.push(res);
                }

                Instr {
                    op: OpCode::LogicalNeg,
                    ..
                } => {
                    let rhs = self.stack.pop().unwrap();
                    let res = Value::unary_op(UnaryOp::LogicalNeg, &rhs)?;
                    self.stack.push(res);
                }

                Instr {
                    op: OpCode::BranchIfTrue,
                    ..
                } => {
                    let cond = self.stack.last().unwrap();
                    if cond.truthy() {
                        self.ip += self.chunk.code[ip].wide_arg();
                    }
                }

                Instr {
                    op: OpCode::BranchIfFalse,
                    ..
                } => {
                    let cond = self.stack.last().unwrap();
                    if !cond.truthy() {
                        self.ip += self.chunk.code[ip].wide_arg();
                    }
                }

                Instr {
                    op: OpCode::Pop, ..
                } => {
                    self.stack.pop().unwrap();
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
