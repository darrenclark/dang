use std::{collections::HashMap, rc::Rc};

use chunk::Chunk;
use function::Function;
use inst::{Instr, OpCode};
use ustr::Ustr;

use crate::{
    ast::{BinOp, UnaryOp},
    interpreter::{exception, Exception},
    value::Value,
};

pub mod chunk;
pub mod disassembler;
pub mod function;
pub mod inst;

pub struct VM {
    stack: Vec<Value>,
    frames: Vec<Frame>,
    globals: HashMap<(Ustr, Ustr), Value>,
}

struct Frame {
    ip: usize,
    base: usize,
    function: Function,
}

impl Frame {
    fn new(function: Function, base: usize) -> Self {
        Frame {
            ip: 0,
            base,
            function,
        }
    }
}

impl VM {
    pub fn new(function: Function) -> Self {
        let frame = Frame::new(function, 0);
        VM {
            stack: Vec::new(),
            frames: vec![frame],
            globals: HashMap::new(),
        }
    }

    pub fn run(&mut self) -> Result<Value, Exception> {
        loop {
            let ip = self.ip();
            *self.ip_mut() += 1;

            match self.chunk().code[ip] {
                Instr {
                    op: OpCode::Constant,
                    arg0,
                    ..
                } => {
                    self.stack
                        .push(self.chunk().constants[arg0 as usize].clone());
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
                    op: OpCode::GlobalIsDefined,
                    arg0,
                    arg1,
                    ..
                } => {
                    let key = (self.read_symbol(arg0), self.read_symbol(arg1));
                    let value = self.globals.contains_key(&key);
                    self.stack.push(Value::Bool(value));
                }

                Instr {
                    op: OpCode::GetLocal,
                    arg0,
                    ..
                } => {
                    let index = self.frames.last().unwrap().base + arg0 as usize;
                    self.stack.push(self.stack[index].clone());
                }

                Instr {
                    op: OpCode::SetLocal,
                    arg0,
                    ..
                } => {
                    let index = self.frames.last().unwrap().base + arg0 as usize;
                    let value = self.stack.pop().unwrap();
                    self.stack[index] = value;
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
                        Value::Function(function) => {
                            self.frames.push(Frame::new(function, self.stack.len()));
                            // TODO: Optimize this, so that we don't split_off args on this code
                            // path & repush them
                            for arg in args {
                                self.stack.push(arg);
                            }
                            continue;
                        }
                        _ => todo!(),
                    };
                    self.stack.push(res);
                }

                Instr {
                    op: OpCode::Return, ..
                } => {
                    if self.frames.len() == 1 {
                        return Ok(self.stack.pop().unwrap());
                    } else {
                        let res = self.stack.pop().unwrap();
                        let frame = self.frames.pop().unwrap();
                        self.stack.truncate(frame.base);
                        self.stack.push(res);
                    }
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
                        *self.ip_mut() += self.chunk().code[ip].wide_arg();
                    }
                }

                Instr {
                    op: OpCode::BranchIfFalse,
                    ..
                } => {
                    let cond = self.stack.last().unwrap();
                    if !cond.truthy() {
                        *self.ip_mut() += self.chunk().code[ip].wide_arg();
                    }
                }

                Instr {
                    op: OpCode::Pop, ..
                } => {
                    self.stack.pop().unwrap();
                }

                Instr {
                    op: OpCode::Jump, ..
                } => {
                    *self.ip_mut() += self.chunk().code[ip].wide_arg();
                }

                instr @ Instr {
                    op: OpCode::MakeList,
                    ..
                } => {
                    let args = self.stack.split_off(self.stack.len() - instr.wide_arg());
                    self.stack.push(Value::List(Rc::new(args)));
                }

                instr @ Instr {
                    op: OpCode::MakeTuple,
                    ..
                } => {
                    let args = self.stack.split_off(self.stack.len() - instr.wide_arg());
                    self.stack.push(Value::Tuple(args));
                }

                instr @ Instr {
                    op: OpCode::MakeDict,
                    ..
                } => {
                    #[allow(clippy::mutable_key_type)]
                    let dict = self
                        .stack
                        .split_off(self.stack.len() - 2 * instr.wide_arg())
                        .chunks(2)
                        .map(|chunk| {
                            let key = chunk[0].clone();
                            let value = chunk[1].clone();
                            (key, value)
                        })
                        .collect();

                    self.stack.push(Value::Dict(dict));
                }

                Instr {
                    op: OpCode::GetField,
                    ..
                } => {
                    let key = self.stack.pop().unwrap();
                    let obj = self.stack.pop().unwrap();
                    let value = obj.at_field(key.unwrap_string())?;
                    self.stack.push(value);
                }

                Instr {
                    op: OpCode::GetSubscript,
                    ..
                } => {
                    let index = self.stack.pop().unwrap();
                    let obj = self.stack.pop().unwrap();
                    let value = obj.at(&index)?;
                    self.stack.push(value);
                }

                Instr {
                    op: OpCode::SetField,
                    ..
                } => {
                    let value = self.stack.pop().unwrap();
                    let key = self.stack.pop().unwrap();
                    let obj = self.stack.last_mut().unwrap();
                    *obj.at_field_mut(key.unwrap_string())? = value;
                }

                Instr {
                    op: OpCode::SetSubscript,
                    ..
                } => {
                    let value = self.stack.pop().unwrap();
                    let index = self.stack.pop().unwrap();
                    let obj = self.stack.last_mut().unwrap();
                    *obj.at_mut(&index)? = value;
                }

                Instr {
                    op: OpCode::Dup,
                    arg0,
                    ..
                } => {
                    let index = self.stack.len() - 1 - arg0 as usize;
                    let value = self.stack[index].clone();
                    self.stack.push(value);
                }
            }
        }
    }

    pub fn read_symbol(&self, index: u8) -> Ustr {
        match &self.chunk().constants[index as usize] {
            Value::Symbol(s) => *s,
            other => panic!("Expected symbol, got {}", other),
        }
    }

    fn chunk(&self) -> &Chunk {
        self.frames.last().unwrap().function.chunk()
    }

    fn ip(&self) -> usize {
        self.frames.last().unwrap().ip
    }

    fn ip_mut(&mut self) -> &mut usize {
        &mut self.frames.last_mut().unwrap().ip
    }
}
