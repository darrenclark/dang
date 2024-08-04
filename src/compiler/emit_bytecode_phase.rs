use crate::{
    ast::{BinOp, Node, NodeKind, UnaryOp},
    interpreter::Exception,
    module::ModuleName,
    native_funcs,
    program::Program,
    scope::VariableAllocation,
    value::Value,
    vm::{chunk::Chunk, function::Function, inst::Instr},
};

use super::{CompilationState, Compiler};

pub fn emit_bytecode_phase(
    _compiler: &Compiler,
    compilation_state: &mut CompilationState,
    program: &mut Program,
) -> Result<(), Exception> {
    let mut chunk = Chunk::new();

    let mut emitter = Emitter {
        compilation_state,
        program,
        chunk: &mut chunk,
        pushed_locals: Vec::new(),
    };
    emitter.emit(compilation_state.ast.as_ref().unwrap());

    compilation_state.chunk = chunk;

    Ok(())
}

struct Emitter<'a> {
    compilation_state: &'a CompilationState,
    program: &'a Program,
    chunk: &'a mut Chunk,
    pushed_locals: Vec<usize>,
}

impl Emitter<'_> {
    fn emit(&mut self, node: &Node) {
        match &node.kind {
            NodeKind::SourceFile(statements) => {
                self.emit_module_load_guard(self.compilation_state.module_name);

                for statement in statements {
                    self.emit(statement);
                }

                let constant = self.chunk.write_constant(Value::Bool(true));
                self.chunk.write(Instr::constant(constant));
                self.chunk.write(Instr::return_());
            }
            NodeKind::Module(_) => {}
            NodeKind::Import(kind) => {
                let module_name = ModuleName::from(kind.module_name());
                let module = self.program.get_module(&module_name).unwrap();
                let function = Value::Function(module.function.clone());
                let constant = self.chunk.write_constant(function);
                self.chunk.write(Instr::constant(constant));
                self.chunk.write(Instr::call(0));
                self.chunk.write(Instr::pop());
            }
            NodeKind::Body(statements) => {
                self.pushed_locals.push(0);

                for statement in statements {
                    self.emit(statement);
                }

                for _ in 0..self.pushed_locals.pop().unwrap() {
                    self.chunk.write(Instr::pop());
                }
            }
            NodeKind::Builtin { identifier } => {
                let ptr = native_funcs::func(identifier.unwrap_identifier()).unwrap();
                let constant = self.chunk.write_constant(Value::NativeFunc(ptr));
                self.chunk.write(Instr::constant(constant));
                self.emit_set(node);
            }
            NodeKind::Let { pattern, expr } | NodeKind::Var { pattern, expr } => {
                self.emit_define(pattern, expr);
            }
            NodeKind::Assignment { pattern, expr } => {
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
            NodeKind::BinaryOp {
                op: BinOp::LogicalOr,
                lhs,
                rhs,
            } => {
                self.emit(lhs);
                let branch = self.chunk.write(Instr::branch_if_true(0));
                self.emit(rhs);
                self.chunk.patch_jump(branch);
            }
            NodeKind::BinaryOp {
                op: BinOp::LogicalAnd,
                lhs,
                rhs,
            } => {
                self.emit(lhs);
                let branch = self.chunk.write(Instr::branch_if_false(0));
                self.emit(rhs);
                self.chunk.patch_jump(branch);
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
                    BinOp::LogicalOr | BinOp::LogicalAnd => unreachable!("handled above"),
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
            NodeKind::If {
                condition,
                body,
                else_branch,
            } => {
                self.emit(condition);
                let branch = self.chunk.write(Instr::branch_if_false(0));

                // body - first pop off condition
                self.chunk.write(Instr::pop());
                self.emit(body);
                let jump = self.chunk.write(Instr::jump(0));

                // else branch - always needs to pop off the condition
                self.chunk.patch_jump(branch);
                self.chunk.write(Instr::pop());
                if let Some(else_branch) = else_branch {
                    self.emit(else_branch);
                }

                self.chunk.patch_jump(jump);
            }
            NodeKind::FunctionLiteral { arg_names, body } => {
                let mut chunk = Chunk::new();

                let mut emitter = Emitter {
                    compilation_state: self.compilation_state,
                    program: self.program,
                    chunk: &mut chunk,
                    pushed_locals: Vec::new(),
                };
                emitter.emit_function(arg_names, body);

                let function = Value::Function(Function::new(chunk));
                let constant = self.chunk.write_constant(function);
                self.chunk.write(Instr::constant(constant));
            }
            NodeKind::ListLiteral(items) => {
                if let Some(v) = node.compile_time_value() {
                    let constant = self.chunk.write_constant(v);
                    self.chunk.write(Instr::constant(constant));
                } else {
                    for item in items {
                        self.emit(item);
                    }
                    self.chunk.write(Instr::make_list(items.len()));
                }
            }
            NodeKind::TupleLiteral(items) => {
                if let Some(v) = node.compile_time_value() {
                    let constant = self.chunk.write_constant(v);
                    self.chunk.write(Instr::constant(constant));
                } else {
                    for item in items {
                        self.emit(item);
                    }
                    self.chunk.write(Instr::make_tuple(items.len()));
                }
            }
            NodeKind::DictLiteral(pairs) => {
                if let Some(v) = node.compile_time_value() {
                    let constant = self.chunk.write_constant(v);
                    self.chunk.write(Instr::constant(constant));
                } else {
                    for (key, value) in pairs {
                        self.emit(key);
                        self.emit(value);
                    }
                    self.chunk.write(Instr::make_dict(pairs.len()));
                }
            }
            NodeKind::FieldAccess { object, key } => {
                self.emit(object);

                let constant = self
                    .chunk
                    .write_constant(Value::String(key.unwrap_identifier().to_owned()));
                self.chunk.write(Instr::constant(constant));

                self.chunk.write(Instr::get_field());
            }
            NodeKind::Subscript { object, key } => {
                self.emit(object);
                self.emit(key);
                self.chunk.write(Instr::get_subscript());
            }
            NodeKind::FieldAssignment {
                variable_ref,
                path,
                expr,
            } => {
                self.emit_get(variable_ref);

                for key in path[0..path.len() - 1].iter() {
                    match &key.kind {
                        NodeKind::FieldAssignmentPathField { key } => {
                            let constant = self
                                .chunk
                                .write_constant(Value::String(key.unwrap_identifier().to_owned()));
                            self.chunk.write(Instr::constant(constant));
                            self.chunk.write(Instr::dup(1));
                            self.chunk.write(Instr::dup(1));
                            self.chunk.write(Instr::get_field());
                        }
                        NodeKind::FieldAssignmentPathSubscript { key } => {
                            self.emit(key);
                            self.chunk.write(Instr::dup(1));
                            self.chunk.write(Instr::dup(1));
                            self.chunk.write(Instr::get_subscript());
                        }
                        _ => unreachable!(),
                    }
                }

                match &path.last().unwrap().kind {
                    NodeKind::FieldAssignmentPathField { key } => {
                        let constant = self
                            .chunk
                            .write_constant(Value::String(key.unwrap_identifier().to_owned()));
                        self.chunk.write(Instr::constant(constant));
                        self.emit(expr);
                        self.chunk.write(Instr::set_field());
                    }
                    NodeKind::FieldAssignmentPathSubscript { key } => {
                        self.emit(key);
                        self.emit(expr);
                        self.chunk.write(Instr::set_subscript());
                    }
                    _ => unreachable!(),
                }

                for key in path[0..path.len() - 1].iter() {
                    match &key.kind {
                        NodeKind::FieldAssignmentPathField { .. } => {
                            self.chunk.write(Instr::set_field());
                        }
                        NodeKind::FieldAssignmentPathSubscript { .. } => {
                            self.chunk.write(Instr::set_subscript());
                        }
                        _ => unreachable!(),
                    }
                }

                self.emit_set(variable_ref);
            }
            NodeKind::For {
                pattern,
                enumerable,
                body,
            } => {
                self.pushed_locals.push(0);

                // get iterator
                // TODO: support structs implementing iter
                let iter_fn = self
                    .chunk
                    .write_constant(Value::NativeFunc(native_funcs::iter));
                self.chunk.write(Instr::constant(iter_fn));
                self.emit(enumerable);
                self.chunk.write(Instr::call(1));
                *self.pushed_locals.last_mut().unwrap() += 1;

                // allocate loop variable
                match &pattern.kind {
                    NodeKind::PatternIdentifier { .. } => {
                        if let VariableAllocation::Global { .. } =
                            self.get_variable_allocation(pattern)
                        {
                            unreachable!("for loop variables will never be global");
                        } else {
                            //let n = self.chunk.write_constant(Value::Nil);
                            //self.chunk.write(Instr::constant(n));
                            // no need to push the nil, since the code later on will leave the
                            // variable in the right stack spot
                            //*self.pushed_locals.last_mut().unwrap() += 1;
                        }
                    }
                    _ => todo!(),
                }

                let loop_start = self.chunk.label("loop_start");

                // call iterator function
                self.emit_get(node);
                self.chunk.write(Instr::call(0));

                // handle result
                self.chunk.write(Instr::check_iter_item());
                let loop_exit_branch = self.chunk.write(Instr::branch_if_false(0));
                // pop off CheckIterItem boolean, leaving the item in the correct stack spot
                self.chunk.write(Instr::pop());

                // loop body
                self.emit(body);
                self.chunk.write(Instr::pop()); // to pop result of body

                // loop back to top - pop local var off the stack first
                self.chunk.write(Instr::pop());
                self.chunk.write_jump_back(loop_start);

                self.chunk.patch_jump(loop_exit_branch);
                // pop of CheckIterItem results
                self.chunk.write(Instr::pop());
                self.chunk.write(Instr::pop());

                // pop off iterator & pattern vars
                for _ in 0..self.pushed_locals.pop().unwrap() {
                    self.chunk.write(Instr::pop());
                }
            }
            ast => todo!("not yet implemented for: {:?}", ast),
        }
    }

    fn emit_define(&mut self, pattern: &Node, expr: &Node) {
        // push value of expr on to stack
        self.emit(expr);

        match &pattern.kind {
            NodeKind::PatternIdentifier { .. } => {
                if let VariableAllocation::Global { .. } = self.get_variable_allocation(pattern) {
                    self.emit_set(pattern);
                } else {
                    *self.pushed_locals.last_mut().unwrap() += 1;
                }
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
        match self.get_variable_allocation(node) {
            VariableAllocation::Global { module, name } => {
                let module = self.chunk.write_constant(Value::Symbol(module.0));
                let name = self.chunk.write_constant(Value::Symbol(name));
                self.chunk.write(Instr::set_global(module, name));
            }
            VariableAllocation::Local { index } => {
                self.chunk.write(Instr::set_local(index as u8));
            }
        }
    }

    fn emit_get(&mut self, node: &Node) {
        match self.get_variable_allocation(node) {
            VariableAllocation::Global { module, name } => {
                let module = self.chunk.write_constant(Value::Symbol(module.0));
                let name = self.chunk.write_constant(Value::Symbol(name));
                self.chunk.write(Instr::get_global(module, name));
            }
            VariableAllocation::Local { index } => {
                self.chunk.write(Instr::get_local(index as u8));
            }
        }
    }

    fn emit_function(&mut self, _arg_names: &[Node], body: &Node) {
        // can't use usual handling of Body because we need to avoid
        // popping locals before the final return
        if let NodeKind::Body(statements) = &body.kind {
            self.pushed_locals.push(0);
            for statement in statements {
                self.emit(statement);
            }
            // ignore number of pushed locals since the return value
            // is at the top of the stack (and the VM will pop to
            // the frame base anyways)
            self.pushed_locals.pop();

            self.chunk.write(Instr::return_());
        }
    }

    fn get_variable_allocation(&self, node: &Node) -> VariableAllocation {
        self.compilation_state
            .variable_allocations
            .get(&node.id)
            .unwrap()
            .clone()
    }

    fn emit_module_load_guard(&mut self, module_name: ModuleName) {
        let module = self.chunk.write_constant(Value::Symbol(module_name.0));
        let name = self.chunk.write_constant(Value::Symbol("<loaded>".into()));
        self.chunk.write(Instr::global_is_defined(module, name));

        let branch = self.chunk.write(Instr::branch_if_false(0));
        self.chunk.write(Instr::return_());

        self.chunk.patch_jump(branch);
        self.chunk.write(Instr::logical_neg());
        self.chunk.write(Instr::set_global(module, name));
    }
}
