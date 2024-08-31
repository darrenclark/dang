use std::{cmp::min, collections::HashMap, rc::Rc};

use chunk::Chunk;
use closure::Closure;
use function::Function;
use inst::{Instr, OpCode};
use stats::StatsCollector;
use upvalue::Upvalue;
use ustr::Ustr;

use crate::{
    ast::{BinOp, UnaryOp},
    compiler::variable::UpvalueSource,
    exception::{exception, Exception},
    module::ModuleName,
    value::Value,
};

pub mod chunk;
pub mod closure;
pub mod disassembler;
pub mod function;
pub mod inst;
pub mod pattern;
mod stats;
pub mod upvalue;

pub struct VM {
    stack: Vec<Value>,
    frames: Vec<Frame>,
    globals: HashMap<(Ustr, Ustr), Value>,
    open_upvalues: Vec<(usize, Upvalue)>,
    argv: Value,
    stats: StatsCollector,
}

struct Frame {
    ip: usize,
    base: usize,
    function: Function,
    upvalues: Vec<Upvalue>,
}

impl Frame {
    fn new(function: Function, base: usize) -> Self {
        Frame {
            ip: 0,
            base,
            function,
            upvalues: Vec::new(),
        }
    }

    fn new_closure(closure: Closure, base: usize) -> Self {
        Frame {
            ip: 0,
            base,
            function: closure.function().clone(),
            upvalues: closure.upvalues().to_vec(),
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
            open_upvalues: Vec::new(),
            argv: Value::Nil,
            stats: StatsCollector::new(),
        }
    }

    pub fn set_argv(&mut self, argv: Vec<String>) {
        self.argv = argv.into();
    }

    pub fn run(&mut self) -> Result<Value, Exception> {
        loop {
            let result = if cfg!(feature = "stats") {
                let mut timer = stats::InstructionTimer::new(self.chunk().code[self.ip()].op);
                let result = self.run_n(1);
                timer.stop(&mut self.stats);
                result
            } else {
                self.run_n(usize::MAX)
            };

            match result {
                Ok(Some(value)) => return Ok(value),
                Ok(None) => {}
                Err(e) => return Err(e),
            }
        }
    }

    fn run_n(&mut self, n: usize) -> Result<Option<Value>, Exception> {
        for _ in 0..n {
            let ip = self.ip();
            let instr = self.chunk().code[ip];

            /*print!("> ");
            disassembler::disassemble_instruction(self.chunk(), ip);
            print!("  ");
            for v in self.stack.iter() {
                print!("{} ", v);
            }
            println!();*/
            *self.ip_mut() += 1;

            //println!("======= ip: {} ========", ip);
            //disassembler::disassemble_instruction(self.chunk(), &self.chunk().code[ip]);

            match instr {
                Instr {
                    op: OpCode::VmArg,
                    arg0: 1,
                    ..
                } => {
                    self.stack.push(self.argv.clone());
                }

                Instr {
                    op: OpCode::VmArg,
                    arg0,
                    ..
                } => {
                    exception!("Unknown VmArg: {}", arg0);
                }

                Instr {
                    op: OpCode::Constant,
                    arg0,
                    ..
                } => {
                    self.stack
                        .push(self.chunk().constants[arg0 as usize].clone());
                }

                Instr {
                    op: OpCode::PushNil,
                    ..
                } => {
                    self.stack.push(Value::Nil);
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
                    op: OpCode::GetUpvalue,
                    arg0,
                    ..
                } => {
                    let index = arg0 as usize;
                    let value = self.frames.last().unwrap().upvalues[index]
                        .get(&self.stack)
                        .clone();
                    self.stack.push(value);
                }

                Instr {
                    op: OpCode::SetUpvalue,
                    arg0,
                    ..
                } => {
                    let index = arg0 as usize;
                    let value = self.stack.last().unwrap().clone();
                    *self.frames.last_mut().unwrap().upvalues[index].get_mut(&mut self.stack) =
                        value;
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
                    self.call(callee, args)?;
                }

                Instr {
                    op: OpCode::TailCall,
                    arg0,
                    ..
                } => {
                    let args = self.stack.split_off(self.stack.len() - arg0 as usize);
                    let callee = self.stack.pop().unwrap();

                    let frame = self.frames.pop().unwrap();
                    self.close_upvalues(frame.base);
                    self.stack.truncate(frame.base);

                    self.call(callee, args)?;
                }

                Instr {
                    op: OpCode::Return, ..
                } => {
                    if self.frames.len() == 1 {
                        return Ok(Some(self.stack.pop().unwrap()));
                    } else {
                        let res = self.stack.pop().unwrap();
                        let frame = self.frames.pop().unwrap();
                        self.close_upvalues(frame.base);
                        self.stack.truncate(frame.base);
                        self.stack.push(res);
                    }
                }

                Instr {
                    op: OpCode::Closure,
                    ..
                } => {
                    let function = match self.stack.pop().unwrap() {
                        Value::Function(function) => function,
                        v => panic!("Expected closure, got {:?}", v),
                    };

                    let upvalues = function
                        .upvalue_sources()
                        .iter()
                        .map(|source| self.create_upvalue(source))
                        .collect();

                    let closure = Closure::new(function, upvalues);

                    self.stack.push(Value::Closure(closure));
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
                    op: OpCode::PopLocals,
                    ..
                } => {
                    let n = self.chunk().code[ip].wide_arg();
                    let res = self.stack.pop().unwrap();
                    self.close_upvalues(self.stack.len() - n);
                    self.stack.truncate(self.stack.len() - n);
                    self.stack.push(res);
                }

                Instr {
                    op: OpCode::Jump, ..
                } => {
                    *self.ip_mut() += self.chunk().code[ip].wide_arg();
                }

                Instr {
                    op: OpCode::JumpBack,
                    ..
                } => {
                    let offset = self.chunk().code[ip].wide_arg();
                    *self.ip_mut() -= offset + 1;
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

                    self.stack.push(Value::dict(dict));
                }

                Instr {
                    op: OpCode::MakeStruct,
                    arg0: kind,
                    arg1: size,
                    ..
                } => {
                    #[allow(clippy::mutable_key_type)]
                    let dict = self
                        .stack
                        .split_off(self.stack.len() - 2 * size as usize)
                        .chunks(2)
                        .map(|chunk| {
                            let key = chunk[0].unwrap_symbol();
                            let value = chunk[1].clone();
                            (key, value)
                        })
                        .collect();

                    let module = ModuleName(self.chunk().constants[kind as usize].unwrap_symbol());

                    self.stack.push(Value::struct_(module, dict));
                }

                Instr {
                    op: OpCode::GetField,
                    ..
                } => {
                    let key = self.stack.pop().unwrap();
                    let obj = self.stack.pop().unwrap();
                    let value = obj.at_field(&key.unwrap_symbol())?;
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
                    *obj.at_field_mut(&key.unwrap_symbol())? = value;
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

                Instr {
                    op: OpCode::GetIter,
                    ..
                } => {
                    let obj = self.stack.pop().unwrap();
                    match &obj {
                        Value::Struct(module_name, _) => {
                            let key = (module_name.0, Ustr::from("__iter__"));
                            if let Some(iter) = self.globals.get(&key) {
                                self.call(iter.clone(), vec![obj])?;
                            } else {
                                exception!("struct `{}` doesn't implement iter", module_name);
                            }
                        }
                        Value::NativeFunc(_) | Value::NativeClosure(_) => {
                            // TODO: figure out how to check arity of native functions
                            self.stack.push(obj);
                        }
                        Value::Function(f) if f.arity() == 0 => {
                            // already (likely) an iterator
                            self.stack.push(obj);
                        }
                        Value::Closure(c) if c.function().arity() == 0 => {
                            // already (likely) an iterator
                            self.stack.push(obj);
                        }
                        _ => {
                            let iter = obj.into_iter_function()?;
                            self.stack.push(iter);
                        }
                    }
                }

                Instr {
                    op: OpCode::CallIter,
                    ..
                } => {
                    let value = self.stack.last().unwrap().clone();
                    self.call(value, vec![])?;
                }

                Instr {
                    op: OpCode::ForIter,
                    ..
                } => {
                    let res = self.stack.pop().unwrap();
                    match res {
                        Value::Nil => {
                            self.stack.push(Value::Nil);
                            *self.ip_mut() += self.chunk().code[ip].wide_arg();
                        }
                        Value::Tuple(t) if t.len() == 2 => {
                            self.stack.push(t[1].clone());
                        }
                        _ => {
                            exception!(
                                "expected tuple with 2 elements or nil from iterator, got {:?}",
                                res
                            )
                        }
                    }
                }

                Instr {
                    op: OpCode::UnpackTuple,
                    arg0,
                    ..
                } => {
                    let tuple = self.stack.pop().unwrap();
                    match tuple {
                        Value::Tuple(t) if t.len() == arg0 as usize => {
                            self.stack.extend(t);
                        }
                        _ => {
                            exception!("expected {} arity tuple, got {}", arg0, tuple);
                        }
                    }
                }

                Instr {
                    op: OpCode::Match,
                    arg0,
                    ..
                } => {
                    let value = self.stack.pop().unwrap();
                    let pattern = &self.chunk().patterns[arg0 as usize];
                    let result = pattern.match_pattern(&value)?;
                    self.stack.extend(result);
                }

                Instr {
                    op: OpCode::TryMatch,
                    arg0,
                    ..
                } => {
                    let top = self.stack.last().unwrap();
                    let pattern = &self.chunk().patterns[arg0 as usize];
                    if let Ok(result) = pattern.match_pattern(top) {
                        self.stack.extend(result);
                    } else {
                        *self.ip_mut() += self.chunk().code[ip].arg1_arg2();
                    }
                }

                Instr {
                    op: OpCode::MatchFailure,
                    ..
                } => {
                    exception!("value didn't match any case: {}", self.stack.pop().unwrap());
                }
            }

            //println!("stack: {:?}", self.stack);
            //println!("=====================");
        }

        Ok(None)
    }

    pub fn read_symbol(&self, index: u8) -> Ustr {
        match &self.chunk().constants[index as usize] {
            Value::Symbol(s) => *s,
            other => panic!("Expected symbol, got {}", other),
        }
    }

    pub fn print_stacktrace(&self, exception: Exception) {
        println!("{} at:", exception);
        self.frames.iter().rev().for_each(|frame| {
            let function = &frame.function;
            println!(
                "  {}:{}:  `{}`",
                function.get_source_file(),
                function.get_source_loc(frame.ip - 1).line,
                function.get_debug_name(),
            );
        });
    }

    pub fn print_stack(&self) {
        println!("stack:");

        for v in self.stack.iter().rev() {
            println!("  {:?}", v);
        }
    }

    pub fn print_disasm(&self) {
        println!("disasm:");
        let context = 10;

        let ip = self.ip();
        let chunk = self.chunk();

        let from = ip.saturating_sub(context);
        let to = min(ip + context, chunk.code.len() - 1);

        for (i, _) in chunk.code.iter().enumerate() {
            if i < from {
                continue;
            }
            if i == ip {
                print!("> ");
            } else {
                print!("  ");
            }
            disassembler::disassemble_instruction(chunk, i);
            if i >= to {
                break;
            }
        }
    }

    pub fn print_stats(&self) {
        self.stats.print();
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

    fn call(&mut self, callee: Value, args: Vec<Value>) -> Result<(), Exception> {
        match callee {
            Value::NativeFunc(ptr) => {
                let res = ptr(&args)?;
                self.stack.push(res);
            }
            Value::NativeClosure(closure) => {
                let res = closure.closure.borrow_mut()(&args)?;
                self.stack.push(res);
            }
            Value::Function(function) => {
                if args.len() != function.arity() {
                    exception!(
                        "expected {} arguments, got {}",
                        function.arity(),
                        args.len()
                    );
                }

                self.frames.push(Frame::new(function, self.stack.len()));
                // TODO: Optimize this, so that we don't split_off args on this code
                // path & repush them
                for arg in args {
                    self.stack.push(arg);
                }
            }
            Value::Closure(closure) => {
                if args.len() != closure.function().arity() {
                    exception!(
                        "expected {} arguments, got {}",
                        closure.function().arity(),
                        args.len()
                    );
                }

                self.frames
                    .push(Frame::new_closure(closure, self.stack.len()));
                // TODO: Optimize this, so that we don't split_off args on this code
                // path & repush them
                for arg in args {
                    self.stack.push(arg);
                }
            }
            _ => exception!("cannot call a non-function: {}", callee),
        }
        Ok(())
    }

    fn create_upvalue(&mut self, source: &UpvalueSource) -> Upvalue {
        match source {
            UpvalueSource::Local {
                stack_index_relative_to_base,
            } => {
                let stack_index = self.frames.last().unwrap().base + *stack_index_relative_to_base;

                self.open_upvalues
                    .iter()
                    .find_map(|(index, upvalue)| {
                        if *index == stack_index {
                            Some(upvalue.clone())
                        } else {
                            None
                        }
                    })
                    .unwrap_or_else(|| {
                        let upvalue = Upvalue::new(stack_index);
                        self.open_upvalues.push((stack_index, upvalue.clone()));

                        upvalue
                    })
            }
            UpvalueSource::Upvalue { upvalue_index } => {
                self.frames.last().unwrap().upvalues[*upvalue_index].clone()
            }
        }
    }

    fn close_upvalues(&mut self, new_stack_len: usize) {
        for (stack_index, upvalue) in &self.open_upvalues {
            if *stack_index >= new_stack_len {
                upvalue.close(self.stack[*stack_index].clone());
            }
        }

        self.open_upvalues
            .retain(|(stack_index, _)| *stack_index < new_stack_len);
    }
}
