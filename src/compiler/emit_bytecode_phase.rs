use std::collections::HashMap;

use ustr::Ustr;

use crate::{
    ast::{BinOp, Node, NodeKind, UnaryOp},
    interpreter::Exception,
    module::ModuleName,
    native_funcs,
    program::Program,
    scope::{UpvalueSource, VariableAllocation},
    value::Value,
    vm::{chunk::Chunk, function::Function, inst::Instr},
};

use super::{
    function_names_phase::FunctionName, resolve_structs_phase::ReferencedStruct, CompilationState,
    Compiler,
};

pub fn emit_bytecode_phase(
    _compiler: &Compiler,
    compilation_state: &mut CompilationState,
    program: &mut Program,
) -> Result<(), Exception> {
    let mut chunk = Chunk::new();
    let mut upvalue_sources = Vec::new();

    let mut emitter = Emitter {
        compilation_state,
        program,
        chunk: &mut chunk,
        upvalue_sources: &mut upvalue_sources,
        pushed_locals: Vec::new(),
    };
    emitter.emit(compilation_state.ast.as_ref().unwrap());

    let name = &compilation_state
        .tags
        .get::<FunctionName>(compilation_state.ast.as_ref().unwrap().id)
        .unwrap()
        .name;
    compilation_state.function = Function::new(chunk, upvalue_sources, name.clone());

    Ok(())
}

struct Emitter<'a> {
    compilation_state: &'a CompilationState,
    program: &'a Program,
    chunk: &'a mut Chunk,
    upvalue_sources: &'a mut Vec<UpvalueSource>,
    pushed_locals: Vec<usize>,
}

impl Emitter<'_> {
    fn emit(&mut self, node: &Node) {
        match &node.kind {
            NodeKind::SourceFile(statements) => {
                self.upvalue_sources.clone_from(
                    self.compilation_state
                        .function_upvalues
                        .get(&node.id)
                        .unwrap(),
                );

                let module_name = self.compilation_state.module_name;

                self.emit_module_load_guard(module_name);

                for (name, value) in &self.compilation_state.constants {
                    let value = self.chunk.write_constant(value.clone());
                    self.chunk.write(Instr::constant(value));

                    let module = self.chunk.write_constant(Value::Symbol(module_name.0));
                    let name = self.chunk.write_constant(Value::Symbol(Ustr::from(name)));
                    self.chunk.write(Instr::set_global(module, name));
                }

                for (i, statement) in statements.iter().enumerate() {
                    self.emit(statement);
                    if i < statements.len() - 1 {
                        self.chunk.write(Instr::pop());
                    }
                }

                self.chunk.write(Instr::return_());
            }
            NodeKind::Module(_) => {
                self.chunk.write(Instr::push_nil());
            }
            NodeKind::Import(kind) => {
                let module_name = ModuleName::from(kind.module_name());
                let module = self.program.get_module(&module_name).unwrap();
                let function = Value::Function(module.function.clone());
                let constant = self.chunk.write_constant(function);
                self.chunk.write(Instr::constant(constant));
                self.chunk.write(Instr::call(0));
            }
            NodeKind::StructDef { .. } => {
                self.chunk.write(Instr::push_nil());
            }
            NodeKind::Body(statements) => {
                self.pushed_locals.push(0);

                for (i, statement) in statements.iter().enumerate() {
                    self.emit(statement);
                    if i < statements.len() - 1 {
                        self.chunk.write(Instr::pop());
                    }
                }

                let locals_count = self.pushed_locals.pop().unwrap();
                if locals_count > 0 {
                    self.chunk.write(Instr::pop_locals(locals_count));
                }
            }
            NodeKind::Builtin { identifier } if identifier.unwrap_identifier() == "argv" => {
                self.emit_vm_arg_func(1, "argv");
                self.emit_set(node);
                self.chunk.write(Instr::push_nil());
            }
            NodeKind::Builtin { identifier } => {
                let ptr = native_funcs::func(identifier.unwrap_identifier()).unwrap();
                let constant = self.chunk.write_constant(Value::NativeFunc(ptr));
                self.chunk.write(Instr::constant(constant));
                self.emit_set(node);
                self.chunk.write(Instr::push_nil());
            }
            NodeKind::Let { pattern, expr } | NodeKind::Var { pattern, expr } => {
                self.emit_define(pattern, expr);
                self.chunk.write(Instr::push_nil());
            }
            NodeKind::Assignment { pattern, expr } => {
                self.emit_assignment(pattern, expr);
                self.chunk.write(Instr::push_nil());
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
                // if lhs is not truthy, pop it off & try rhs
                self.chunk.write(Instr::pop());
                self.emit(rhs);
                // else skip over rhs and leave lhs at top of stack
                self.chunk.patch_jump(branch);
            }
            NodeKind::BinaryOp {
                op: BinOp::LogicalAnd,
                lhs,
                rhs,
            } => {
                self.emit(lhs);
                let branch = self.chunk.write(Instr::branch_if_false(0));
                // if lhs truthy, pop it off & try rhs
                self.chunk.write(Instr::pop());
                self.emit(rhs);
                // else skip over rhs and leave lhs at top of stack
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
                self.emit(function);
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
                } else {
                    self.chunk.write(Instr::push_nil());
                }

                self.chunk.patch_jump(jump);
            }
            NodeKind::FunctionLiteral { arg_names, body } => {
                let mut chunk = Chunk::new();
                let mut upvalue_sources = Vec::new();

                let mut emitter = Emitter {
                    compilation_state: self.compilation_state,
                    program: self.program,
                    chunk: &mut chunk,
                    upvalue_sources: &mut upvalue_sources,
                    pushed_locals: Vec::new(),
                };
                emitter.emit_function(arg_names, body);

                // TODO: refactor this
                upvalue_sources.clone_from(
                    self.compilation_state
                        .function_upvalues
                        .get(&node.id)
                        .unwrap(),
                );

                let has_upvalues = !upvalue_sources.is_empty();

                let function = Value::Function(Function::new(
                    chunk,
                    upvalue_sources,
                    self.function_name(node),
                ));
                let constant = self.chunk.write_constant(function);
                self.chunk.write(Instr::constant(constant));

                if has_upvalues {
                    self.chunk.write(Instr::closure());
                }
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
            NodeKind::StructLiteral { module: _, fields } => {
                // TODO: optimize to emit a compile time value when possible?
                // TODO: optimize to only emit values
                let struct_info = &self
                    .compilation_state
                    .tags
                    .get::<ReferencedStruct>(node.id)
                    .unwrap()
                    .struct_info;

                let module_name = self
                    .chunk
                    .write_constant(Value::Symbol(struct_info.module_name.0));

                let fields_map = fields
                    .iter()
                    .map(|(k, v)| (Ustr::from(k.unwrap_identifier()), v))
                    .collect::<HashMap<_, _>>();

                for field in &struct_info.fields {
                    let key = self.chunk.write_constant(Value::Symbol(*field));
                    self.chunk.write(Instr::constant(key));

                    if let Some(value) = fields_map.get(field) {
                        self.emit(value);
                    } else {
                        let default_value = struct_info.default_values.get(field).unwrap();
                        let constant = self.chunk.write_constant(default_value.clone());
                        self.chunk.write(Instr::constant(constant));
                    }
                }

                self.chunk
                    .write(Instr::make_struct(module_name, struct_info.fields.len()));
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
                self.emit(enumerable);
                self.chunk.write(Instr::get_iter());

                // allocate loop variable (+1 for the result from iterator)
                *self.pushed_locals.last_mut().unwrap() += 1;

                let is_complex_pattern =
                    !matches!(&pattern.kind, NodeKind::PatternIdentifier { .. });
                let pattern_locals = self.count_pattern_locals(pattern);

                let loop_start = self.chunk.label("loop_start");

                // call iterator function
                self.chunk.write(Instr::call_iter());

                // handle result
                let loop_exit_branch = self.chunk.write(Instr::for_iter(0));
                if is_complex_pattern {
                    // preallocate room on stack for pattern
                    for _ in 0..pattern_locals {
                        self.chunk.write(Instr::push_nil());
                        // don't incremented pushed_locals, as we clean these locals up
                        // before the next iteration (and hence won't be there when we exit)
                    }
                    // push & unpack the expression
                    self.chunk.write(Instr::dup(pattern_locals as u8));
                    self.emit_assignment_pattern(pattern);
                }
                // else: (value is correct in local var slot)

                // loop body
                self.emit(body);
                self.chunk.write(Instr::pop()); // to pop result of body

                // loop back to top - pop local var + any pattern locals off the stack first
                self.chunk.write(Instr::pop());
                if is_complex_pattern {
                    for _ in 0..pattern_locals {
                        self.chunk.write(Instr::pop());
                    }
                }
                self.chunk.write_jump_back(loop_start);

                self.chunk.patch_jump(loop_exit_branch);

                // pop off iterator & pattern vars
                for _ in 0..self.pushed_locals.pop().unwrap() {
                    self.chunk.write(Instr::pop());
                }
                self.chunk.write(Instr::push_nil());
            }
            ast => todo!("not yet implemented for: {:?}", ast),
        }
    }

    fn emit_define(&mut self, pattern: &Node, expr: &Node) {
        match &pattern.kind {
            NodeKind::PatternIdentifier { .. } => {
                // simplest case: let x = 123
                if let VariableAllocation::Global { .. } = self.get_variable_allocation(pattern) {
                    self.emit(expr);
                    self.emit_set(pattern);
                } else {
                    self.emit(expr);
                    *self.pushed_locals.last_mut().unwrap() += 1;
                }
            }
            _ => {
                // preallocate room on stack for pattern
                for _ in 0..self.count_pattern_locals(pattern) {
                    self.chunk.write(Instr::push_nil());
                    *self.pushed_locals.last_mut().unwrap() += 1;
                }
                // push & unpack the expression
                self.emit(expr);
                self.emit_assignment_pattern(pattern);
            }
        }
    }

    fn count_pattern_locals(&self, pattern: &Node) -> usize {
        match &pattern.kind {
            NodeKind::PatternIdentifier { .. } => {
                if let VariableAllocation::Local { .. } = self.get_variable_allocation(pattern) {
                    1
                } else {
                    0
                }
            }
            NodeKind::PatternTuple { elements } => {
                elements.iter().map(|e| self.count_pattern_locals(e)).sum()
            }
            _ => unreachable!(),
        }
    }

    fn emit_assignment(&mut self, pattern: &Node, expr: &Node) {
        // push value of expr on to stack
        self.emit(expr);
        self.emit_assignment_pattern(pattern);
    }

    fn emit_assignment_pattern(&mut self, pattern: &Node) {
        match &pattern.kind {
            NodeKind::PatternIdentifier { .. } => {
                self.emit_set(pattern);
            }
            NodeKind::PatternTuple { elements } => {
                self.chunk.write(Instr::unpack_tuple(elements.len()));
                elements.iter().rev().for_each(|element| {
                    self.emit_assignment_pattern(element);
                });
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
            VariableAllocation::Upvalue {
                source: _,
                upvalue_index,
            } => {
                self.chunk.write(Instr::set_upvalue(upvalue_index as u8));
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
            VariableAllocation::Upvalue {
                source: _,
                upvalue_index,
            } => {
                self.chunk.write(Instr::get_upvalue(upvalue_index as u8));
            }
            VariableAllocation::Local { index } => {
                self.chunk.write(Instr::get_local(index as u8));
            }
        }
    }

    fn emit_function(&mut self, _arg_names: &[Node], body: &Node) {
        self.emit(body);
        self.chunk.write(Instr::return_());
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

    fn emit_vm_arg_func(&mut self, arg: u8, name: &str) {
        let mut chunk = Chunk::new();
        chunk.write(Instr::vm_arg(arg));
        chunk.write(Instr::return_());

        let function = Value::Function(Function::new(chunk, Vec::new(), name.to_owned()));
        let constant = self.chunk.write_constant(function);
        self.chunk.write(Instr::constant(constant));
    }

    fn function_name(&self, node: &Node) -> String {
        self.compilation_state
            .tags
            .get::<FunctionName>(node.id)
            .unwrap()
            .name
            .to_string()
    }
}
