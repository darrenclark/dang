use crate::{
    ast::{BinOp, Node, NodeKind, UnaryOp},
    interpreter::Exception,
    native_funcs,
    program::Program,
    scope::VariableLocation,
    value::Value,
    vm::{chunk::Chunk, inst::Instr},
};

use super::{CompilationState, Compiler};

pub fn emit_bytecode_phase(
    _compiler: &Compiler,
    compilation_state: &mut CompilationState,
    _program: &mut Program,
) -> Result<(), Exception> {
    let mut chunk = Chunk::new();

    let mut emitter = Emitter {
        compilation_state,
        chunk: &mut chunk,
    };
    emitter.emit(compilation_state.ast.as_ref().unwrap());

    compilation_state.chunk = chunk;

    Ok(())
}

struct Emitter<'a> {
    compilation_state: &'a CompilationState,
    chunk: &'a mut Chunk,
}

impl Emitter<'_> {
    fn emit(&mut self, node: &Node) {
        match &node.kind {
            NodeKind::SourceFile(statements) => {
                for statement in statements {
                    self.emit(statement);
                }
                self.chunk.write(Instr::return_());
            }
            NodeKind::Builtin { identifier } => {
                let ptr = native_funcs::func(identifier.unwrap_identifier()).unwrap();
                let constant = self.chunk.write_constant(Value::NativeFunc(ptr));
                self.chunk.write(Instr::constant(constant));
                self.emit_set(node);
            }
            NodeKind::Let { pattern, expr }
            | NodeKind::Var { pattern, expr }
            | NodeKind::Assignment { pattern, expr } => {
                self.emit_assignment(pattern, expr);
            }
            NodeKind::NilLiteral => {
                let constant = self.chunk.write_constant(Value::Nil);
                self.chunk.write(Instr::constant(constant));
            }
            NodeKind::BoolLiteral(bool) => {
                let constant = self.chunk.write_constant(Value::Bool(*bool));
                self.chunk.write(Instr::constant(constant));
            }
            NodeKind::StringLiteral(string) => {
                let constant = self.chunk.write_constant(Value::String(string.clone()));
                self.chunk.write(Instr::constant(constant));
            }
            NodeKind::IntegerLiteral(int) => {
                let constant = self.chunk.write_constant(Value::Integer(*int));
                self.chunk.write(Instr::constant(constant));
            }
            NodeKind::VariableRef { .. } => {
                self.emit_get(node);
            }
            NodeKind::BinaryOp { op, lhs, rhs } => {
                self.emit(lhs);
                self.emit(rhs);
                match op {
                    BinOp::Add => {
                        self.chunk.write(Instr::add());
                    }
                    BinOp::Sub => {
                        self.chunk.write(Instr::sub());
                    }
                    BinOp::Mul => {
                        self.chunk.write(Instr::mul());
                    }
                    BinOp::Div => {
                        self.chunk.write(Instr::div());
                    }
                    BinOp::Gt => {
                        self.chunk.write(Instr::gt());
                    }
                    BinOp::Gte => {
                        self.chunk.write(Instr::gte());
                    }
                    BinOp::Lt => {
                        self.chunk.write(Instr::lt());
                    }
                    BinOp::Lte => {
                        self.chunk.write(Instr::lte());
                    }
                    BinOp::Eq => {
                        self.chunk.write(Instr::eq());
                    }
                    BinOp::Neq => {
                        self.chunk.write(Instr::neq());
                    }
                    BinOp::LogicalOr => {
                        self.chunk.write(Instr::logical_or());
                    }
                    BinOp::LogicalAnd => {
                        self.chunk.write(Instr::logical_and());
                    }
                }
            }
            NodeKind::UnaryOp { op, rhs } => {
                self.emit(rhs);
                match op {
                    UnaryOp::Neg => {
                        self.chunk.write(Instr::neg());
                    }
                    UnaryOp::LogicalNeg => {
                        self.chunk.write(Instr::logical_neg());
                    }
                }
            }
            NodeKind::FunctionCall { function, args } => {
                self.emit_get(function);
                for arg in args {
                    self.emit(arg);
                }
                self.chunk.write(Instr::call(args.len() as u8));
            }
            _ => todo!(),
        }
    }

    fn emit_assignment(&mut self, pattern: &Node, expr: &Node) {
        // push value of expr on to stack
        self.emit(expr);

        match &pattern.kind {
            NodeKind::PatternIdentifier { .. } => {
                self.emit_set(pattern);
            }
            _ => todo!(),
        }
    }

    fn emit_set(&mut self, node: &Node) {
        let variable_location = self
            .compilation_state
            .variable_locations
            .get(&node.id)
            .unwrap();

        match variable_location {
            VariableLocation::Global { module, name } => {
                let module = self.chunk.write_constant(Value::Symbol(module.0));
                let name = self.chunk.write_constant(Value::Symbol(*name));
                self.chunk.write(Instr::set_global(module, name));
            }
            _ => todo!(),
        }
    }

    fn emit_get(&mut self, node: &Node) {
        let variable_location = self
            .compilation_state
            .variable_locations
            .get(&node.id)
            .unwrap();
        match variable_location {
            VariableLocation::Global { module, name } => {
                let module = self.chunk.write_constant(Value::Symbol(module.0));
                let name = self.chunk.write_constant(Value::Symbol(*name));
                self.chunk.write(Instr::get_global(module, name));
            }
            _ => todo!(),
        }
    }
}
