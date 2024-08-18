use std::collections::HashMap;

use ustr::Ustr;

use crate::{
    ast::{BinOp, Node, NodeKind, UnaryOp},
    exception::Exception,
    line_col::LineCol,
    module::ModuleName,
    native_funcs,
    program::Program,
    scope::Scope,
    value::Value,
    vm::{chunk::Chunk, function::Function, inst::Instr},
};

use super::{
    function_names_phase::FunctionName,
    resolve_structs_phase::ReferencedStruct,
    resolve_variables_phase::ModuleReference,
    tag_return_exprs_phase::IsReturnExpr,
    variable::{UpvalueSource, VariableAllocation, VariableRef},
    CompilationState, Compiler,
};

pub fn emit_bytecode_phase(
    _compiler: &Compiler,
    compilation_state: &mut CompilationState,
    program: &mut Program,
) -> Result<(), Exception> {
    let mut chunk = Chunk::new();
    let mut upvalue_sources = Vec::new();

    // TODO: use actual source file name eventually
    chunk.source_file = compilation_state.module_name.0.clone().to_owned();

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
    compilation_state.function = Function::new(chunk, 0, upvalue_sources, name.clone());

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
                self.upvalue_sources
                    .clone_from(&self.get_upvalue_sources(node));

                let module_name = self.compilation_state.module_name;

                self.emit_module_load_guard(self.compilation_state.module_name, node);

                // TODO: Should these be set as globals? Or can they be inlined where
                // they're used at compile time?
                for (name, value) in &self.compilation_state.constants {
                    let value = self.chunk.write_constant(value.clone());
                    self.write(Instr::constant(value), node);

                    let module = self.chunk.write_constant(Value::Symbol(module_name.0));
                    let name = self.chunk.write_constant(Value::Symbol(Ustr::from(name)));
                    self.write(Instr::set_global(module, name), node);
                }

                for (i, statement) in statements.iter().enumerate() {
                    self.emit(statement);
                    if i < statements.len() - 1 {
                        self.write(Instr::pop(), node);
                    }
                }

                self.write(Instr::return_(), node);
            }
            NodeKind::Module(_) => {
                self.write(Instr::push_nil(), node);
            }
            NodeKind::Import(kind) => {
                let module_name = ModuleName::from(kind.module_name());
                let module = self.program.get_module(&module_name).unwrap();
                let function = Value::Function(module.function.clone());
                let constant = self.chunk.write_constant(function);
                self.write(Instr::constant(constant), node);
                self.write(Instr::call(0), node);
            }
            NodeKind::StructDef { .. } => {
                self.write(Instr::push_nil(), node);
            }
            NodeKind::Body(statements) if statements.is_empty() => {
                self.write(Instr::push_nil(), node);
            }
            NodeKind::Body(statements) => {
                self.pushed_locals.push(0);

                for (i, statement) in statements.iter().enumerate() {
                    self.emit(statement);
                    if i < statements.len() - 1 {
                        self.write(Instr::pop(), node);
                    }
                }

                let locals_count = self.pushed_locals.pop().unwrap();
                if locals_count > 0 {
                    self.write(Instr::pop_locals(locals_count), node);
                }
            }
            NodeKind::Builtin { identifier } if identifier.unwrap_identifier() == "argv" => {
                let constant = self.vm_arg_func_constant(1, "argv");
                self.write(Instr::constant(constant), node);
                self.emit_set(node);
                self.write(Instr::push_nil(), node);
            }
            NodeKind::Builtin { identifier } if identifier.unwrap_identifier() == "iter" => {
                let constant = self.iter_func_constant();
                self.write(Instr::constant(constant), node);
                self.emit_set(node);
                self.write(Instr::push_nil(), node);
            }
            NodeKind::Builtin { identifier } => {
                let ptr = native_funcs::func(identifier.unwrap_identifier()).unwrap();
                let constant = self.chunk.write_constant(Value::NativeFunc(ptr));
                self.write(Instr::constant(constant), node);
                self.emit_set(node);
                self.write(Instr::push_nil(), node);
            }
            NodeKind::Let { pattern, expr } | NodeKind::Var { pattern, expr } => {
                self.emit_define(pattern, expr);
                self.write(Instr::push_nil(), node);
            }
            NodeKind::Assignment { pattern, expr } => {
                self.emit_assignment(pattern, expr);
                self.write(Instr::push_nil(), node);
            }
            NodeKind::NilLiteral => {
                let constant = self.chunk.write_constant(Value::Nil);
                self.write(Instr::constant(constant), node);
            }
            NodeKind::BoolLiteral(bool) => {
                let constant = self.chunk.write_constant(Value::Bool(*bool));
                self.write(Instr::constant(constant), node);
            }
            NodeKind::StringLiteral(string) => {
                let constant = self.chunk.write_constant(Value::string(string));
                self.write(Instr::constant(constant), node);
            }
            NodeKind::IntegerLiteral(int) => {
                let constant = self.chunk.write_constant(Value::Integer(*int));
                self.write(Instr::constant(constant), node);
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
                let branch = self.write(Instr::branch_if_true(0), node);
                // if lhs is not truthy, pop it off & try rhs
                self.write(Instr::pop(), node);
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
                let branch = self.write(Instr::branch_if_false(0), node);
                // if lhs truthy, pop it off & try rhs
                self.write(Instr::pop(), node);
                self.emit(rhs);
                // else skip over rhs and leave lhs at top of stack
                self.chunk.patch_jump(branch);
            }
            NodeKind::BinaryOp { op, lhs, rhs } => {
                self.emit(lhs);
                self.emit(rhs);
                match op {
                    BinOp::Add => {
                        self.write(Instr::add(), node);
                    }
                    BinOp::Sub => {
                        self.write(Instr::sub(), node);
                    }
                    BinOp::Mul => {
                        self.write(Instr::mul(), node);
                    }
                    BinOp::Div => {
                        self.write(Instr::div(), node);
                    }
                    BinOp::Gt => {
                        self.write(Instr::gt(), node);
                    }
                    BinOp::Gte => {
                        self.write(Instr::gte(), node);
                    }
                    BinOp::Lt => {
                        self.write(Instr::lt(), node);
                    }
                    BinOp::Lte => {
                        self.write(Instr::lte(), node);
                    }
                    BinOp::Eq => {
                        self.write(Instr::eq(), node);
                    }
                    BinOp::Neq => {
                        self.write(Instr::neq(), node);
                    }
                    BinOp::LogicalOr | BinOp::LogicalAnd => unreachable!("handled above"),
                }
            }
            NodeKind::UnaryOp { op, rhs } => {
                self.emit(rhs);
                match op {
                    UnaryOp::Neg => {
                        self.write(Instr::neg(), node);
                    }
                    UnaryOp::LogicalNeg => {
                        self.write(Instr::logical_neg(), node);
                    }
                }
            }
            NodeKind::FunctionCall { function, args } => {
                self.emit(function);
                for arg in args {
                    self.emit(arg);
                }

                if self.is_tail_call(node) {
                    self.write(Instr::tail_call(args.len() as u8), node);
                } else {
                    self.write(Instr::call(args.len() as u8), node);
                }
            }
            NodeKind::If {
                condition,
                body,
                else_branch,
            } => {
                self.emit(condition);
                let branch = self.write(Instr::branch_if_false(0), node);

                // body - first pop off condition
                self.write(Instr::pop(), node);
                self.emit(body);
                let jump = self.write(Instr::jump(0), node);

                // else branch - always needs to pop off the condition
                self.chunk.patch_jump(branch);
                self.write(Instr::pop(), node);
                if let Some(else_branch) = else_branch {
                    self.emit(else_branch);
                } else {
                    self.write(Instr::push_nil(), node);
                }

                self.chunk.patch_jump(jump);
            }
            NodeKind::FunctionLiteral { arg_names, body } => {
                let mut chunk = Chunk::new();
                let mut upvalue_sources = self.get_upvalue_sources(node);

                // TODO: use actual source file name eventually
                chunk.source_file = self.compilation_state.module_name.0.clone().to_owned();

                let mut emitter = Emitter {
                    compilation_state: self.compilation_state,
                    program: self.program,
                    chunk: &mut chunk,
                    upvalue_sources: &mut upvalue_sources,
                    pushed_locals: Vec::new(),
                };
                emitter.emit_function(arg_names, body);

                let has_upvalues = !upvalue_sources.is_empty();

                let function = Value::Function(Function::new(
                    chunk,
                    arg_names.len(),
                    upvalue_sources,
                    self.function_name(node),
                ));
                let constant = self.chunk.write_constant(function);
                self.write(Instr::constant(constant), node);

                if has_upvalues {
                    self.write(Instr::closure(), node);
                }
            }
            NodeKind::ListLiteral(items) => {
                if let Some(v) = node.compile_time_value() {
                    let constant = self.chunk.write_constant(v);
                    self.write(Instr::constant(constant), node);
                } else {
                    for item in items {
                        self.emit(item);
                    }
                    self.write(Instr::make_list(items.len()), node);
                }
            }
            NodeKind::TupleLiteral(items) => {
                if let Some(v) = node.compile_time_value() {
                    let constant = self.chunk.write_constant(v);
                    self.write(Instr::constant(constant), node);
                } else {
                    for item in items {
                        self.emit(item);
                    }
                    self.write(Instr::make_tuple(items.len()), node);
                }
            }
            NodeKind::DictLiteral(pairs) => {
                if let Some(v) = node.compile_time_value() {
                    let constant = self.chunk.write_constant(v);
                    self.write(Instr::constant(constant), node);
                } else {
                    for (key, value) in pairs {
                        self.emit(key);
                        self.emit(value);
                    }
                    self.write(Instr::make_dict(pairs.len()), node);
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
                    self.write(Instr::constant(key), node);

                    if let Some(value) = fields_map.get(field) {
                        self.emit(value);
                    } else {
                        let default_value = struct_info.default_values.get(field).unwrap();
                        let constant = self.chunk.write_constant(default_value.clone());
                        self.write(Instr::constant(constant), node);
                    }
                }

                self.write(
                    Instr::make_struct(module_name, struct_info.fields.len()),
                    node,
                );
            }
            NodeKind::FieldAccess { object, key } if self.is_module_ref(object) => {
                let module_name = self
                    .compilation_state
                    .tags
                    .get::<ModuleReference>(object.id)
                    .unwrap()
                    .module_name;

                let name = key.unwrap_identifier();

                let m = self.chunk.write_constant(Value::Symbol(module_name.0));
                let n = self.chunk.write_constant(Value::Symbol(Ustr::from(name)));
                self.write(Instr::get_global(m, n), node);
            }
            NodeKind::FieldAccess { object, key } => {
                self.emit(object);

                let constant = self
                    .chunk
                    .write_constant(Value::string(key.unwrap_identifier()));
                self.write(Instr::constant(constant), node);

                self.write(Instr::get_field(), node);
            }
            NodeKind::Subscript { object, key } => {
                self.emit(object);
                self.emit(key);
                self.write(Instr::get_subscript(), node);
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
                                .write_constant(Value::string(key.unwrap_identifier()));
                            self.write(Instr::constant(constant), node);
                            self.write(Instr::dup(1), node);
                            self.write(Instr::dup(1), node);
                            self.write(Instr::get_field(), node);
                        }
                        NodeKind::FieldAssignmentPathSubscript { key } => {
                            self.emit(key);
                            self.write(Instr::dup(1), node);
                            self.write(Instr::dup(1), node);
                            self.write(Instr::get_subscript(), node);
                        }
                        _ => unreachable!(),
                    }
                }

                match &path.last().unwrap().kind {
                    NodeKind::FieldAssignmentPathField { key } => {
                        let constant = self
                            .chunk
                            .write_constant(Value::string(key.unwrap_identifier()));
                        self.write(Instr::constant(constant), node);
                        self.emit(expr);
                        self.write(Instr::set_field(), node);
                    }
                    NodeKind::FieldAssignmentPathSubscript { key } => {
                        self.emit(key);
                        self.emit(expr);
                        self.write(Instr::set_subscript(), node);
                    }
                    _ => unreachable!(),
                }

                for key in path[0..path.len() - 1].iter().rev() {
                    match &key.kind {
                        NodeKind::FieldAssignmentPathField { .. } => {
                            self.write(Instr::set_field(), node);
                        }
                        NodeKind::FieldAssignmentPathSubscript { .. } => {
                            self.write(Instr::set_subscript(), node);
                        }
                        _ => unreachable!(),
                    }
                }

                self.emit_set(variable_ref);
                self.write(Instr::push_nil(), node);
            }
            NodeKind::For {
                pattern,
                enumerable,
                body,
            } => {
                // get iterator
                self.emit(enumerable);
                self.write(Instr::get_iter(), node);

                let is_complex_pattern =
                    !matches!(&pattern.kind, NodeKind::PatternIdentifier { .. });
                let pattern_locals = self.count_pattern_locals(pattern);

                let loop_start = self.chunk.label("loop_start");

                // call iterator function
                self.write(Instr::call_iter(), node);

                // handle result
                let loop_exit_branch = self.write(Instr::for_iter(0), node);
                if is_complex_pattern {
                    // preallocate room on stack for pattern
                    for _ in 0..pattern_locals {
                        self.write(Instr::push_nil(), node);
                    }
                    // push & unpack the expression
                    self.write(Instr::dup(pattern_locals as u8), node);
                    self.emit_assignment_pattern(pattern);
                }
                // else: (value is correct in local var slot)

                // loop body
                self.emit(body);

                // pop local var & any pattern locals off the stack
                let mut locals_to_pop = 1;
                if is_complex_pattern {
                    locals_to_pop += pattern_locals;
                }
                self.write(Instr::pop_locals(locals_to_pop), node);

                // pop result of body (preserved by pop_locals)
                self.write(Instr::pop(), node);

                // finally loop back to start
                self.chunk
                    .write_jump_back(loop_start, node.source.end.clone());

                self.chunk.patch_jump(loop_exit_branch);

                // pop local var slot (always nil here) & iterator
                self.write(Instr::pop(), node);
                self.write(Instr::pop(), node);
                // all for loops evaluate to nil
                self.write(Instr::push_nil(), node);
            }
            ast => todo!("not yet implemented for: {:?}", ast),
        }
    }

    fn write(&mut self, instr: Instr, node: &Node) -> usize {
        self.chunk.write(instr, node.source.start.clone())
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
                    self.write(Instr::push_nil(), pattern);
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
            NodeKind::PatternWildcard => 0,
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
                self.write(Instr::unpack_tuple(elements.len()), pattern);
                elements.iter().rev().for_each(|element| {
                    self.emit_assignment_pattern(element);
                });
            }
            NodeKind::PatternWildcard => {
                self.write(Instr::pop(), pattern);
            }
            _ => todo!(),
        }
    }

    fn emit_set(&mut self, node: &Node) {
        match self.get_variable_allocation(node) {
            VariableAllocation::Global { module, name } => {
                let module = self.chunk.write_constant(Value::Symbol(module.0));
                let name = self.chunk.write_constant(Value::Symbol(name));
                self.write(Instr::set_global(module, name), node);
            }
            VariableAllocation::Upvalue {
                source: _,
                upvalue_index,
            } => {
                self.write(Instr::set_upvalue(upvalue_index as u8), node);
            }
            VariableAllocation::Local { index } => {
                self.write(Instr::set_local(index as u8), node);
            }
        }
    }

    fn emit_get(&mut self, node: &Node) {
        match self.get_variable_allocation(node) {
            VariableAllocation::Global { module, name } => {
                let module = self.chunk.write_constant(Value::Symbol(module.0));
                let name = self.chunk.write_constant(Value::Symbol(name));
                self.write(Instr::get_global(module, name), node);
            }
            VariableAllocation::Upvalue {
                source: _,
                upvalue_index,
            } => {
                self.write(Instr::get_upvalue(upvalue_index as u8), node);
            }
            VariableAllocation::Local { index } => {
                self.write(Instr::get_local(index as u8), node);
            }
        }
    }

    fn emit_function(&mut self, _arg_names: &[Node], body: &Node) {
        self.emit(body);
        self.chunk.write(Instr::return_(), body.source.end.clone());
    }

    fn get_variable_allocation(&self, node: &Node) -> VariableAllocation {
        self.compilation_state
            .tags
            .get::<VariableRef>(node.id)
            .unwrap()
            .allocation
            .clone()
    }

    fn get_upvalue_sources(&self, function_node: &Node) -> Vec<UpvalueSource> {
        self.compilation_state
            .tags
            .get::<Scope>(function_node.id)
            .unwrap()
            .get_upvalue_sources()
    }

    fn emit_module_load_guard(&mut self, module_name: ModuleName, source_file_node: &Node) {
        let module = self.chunk.write_constant(Value::Symbol(module_name.0));
        let name = self.chunk.write_constant(Value::Symbol("<loaded>".into()));
        self.write(Instr::global_is_defined(module, name), source_file_node);

        let branch = self.write(Instr::branch_if_false(0), source_file_node);
        self.write(Instr::return_(), source_file_node);

        self.chunk.patch_jump(branch);
        self.write(Instr::logical_neg(), source_file_node);
        self.write(Instr::set_global(module, name), source_file_node);
    }

    fn vm_arg_func_constant(&mut self, arg: u8, name: &str) -> u8 {
        let mut chunk = Chunk::new();
        chunk.write(Instr::vm_arg(arg), LineCol::unknown());
        chunk.write(Instr::return_(), LineCol::unknown());

        let function = Value::Function(Function::new(chunk, 0, Vec::new(), name.to_owned()));
        self.chunk.write_constant(function)
    }

    fn iter_func_constant(&mut self) -> u8 {
        let mut chunk = Chunk::new();
        chunk.write(Instr::get_iter(), LineCol::unknown());
        chunk.write(Instr::return_(), LineCol::unknown());
        let function = Value::Function(Function::new(chunk, 1, Vec::new(), "iter".to_owned()));
        self.chunk.write_constant(function)
    }

    fn function_name(&self, node: &Node) -> String {
        self.compilation_state
            .tags
            .get::<FunctionName>(node.id)
            .unwrap()
            .name
            .to_string()
    }

    fn is_module_ref(&self, node: &Node) -> bool {
        self.compilation_state
            .tags
            .get::<ModuleReference>(node.id)
            .is_some()
    }

    fn is_tail_call(&self, function_call: &Node) -> bool {
        self.compilation_state
            .tags
            .get::<IsReturnExpr>(function_call.id)
            .map(|is_return_expr| {
                // can't tail call from the root "function"
                is_return_expr.function != self.compilation_state.ast.as_ref().unwrap().id
            })
            .unwrap_or(false)
    }
}
