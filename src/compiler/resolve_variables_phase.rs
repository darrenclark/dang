use std::collections::{HashMap, HashSet};

use ustr::Ustr;

use crate::{
    ast::{AstWalker, ImportKind, Node, NodeId, NodeKind},
    interpreter::{exception, Exception},
    module::ModuleName,
    program::Program,
    scope::{Scope, UpvalueSource, VariableAllocation, VariableLocation},
    value::Value,
};

use super::{CompilationState, Compiler};

#[derive(Debug)]
pub struct ModuleReference {
    pub module_name: ModuleName,
}

pub fn resolve_variables_phase(
    _compiler: &Compiler,
    compilation_state: &mut CompilationState,
    program: &mut Program,
) -> Result<(), Exception> {
    let variable_locations: HashMap<NodeId, VariableLocation>;
    let variable_allocations: HashMap<NodeId, VariableAllocation>;
    let exports: HashMap<String, VariableLocation>;
    let constants: HashMap<String, Value>;
    let closed_over_variables: HashSet<NodeId>;
    let function_upvalues: HashMap<NodeId, Vec<UpvalueSource>>;

    {
        let mut phase = ResolveVariablesPhase::new(program, compilation_state);
        phase.run();
        if !phase.errors.is_empty() {
            exception!("unable to resolve all variables")
        }

        variable_locations = phase.variable_locations;
        variable_allocations = phase.variable_allocations;
        exports = phase.exports;
        constants = phase.constants;
        closed_over_variables = phase.closed_over_variables;
        function_upvalues = phase.function_upvalues;
    }

    compilation_state
        .variable_locations
        .clone_from(&variable_locations);
    compilation_state
        .variable_allocations
        .clone_from(&variable_allocations);
    compilation_state.exports.clone_from(&exports);
    compilation_state.constants.clone_from(&constants);
    compilation_state
        .closed_over_variables
        .clone_from(&closed_over_variables);
    compilation_state
        .function_upvalues
        .clone_from(&function_upvalues);
    Ok(())
}

#[derive(Debug)]
pub struct ResolveVariablesPhase<'a> {
    program: &'a mut Program,
    compilation_state: &'a mut CompilationState,
    pub exports: HashMap<String, VariableLocation>,
    pub constants: HashMap<String, Value>,
    pub variable_locations: HashMap<NodeId, VariableLocation>,
    pub variable_allocations: HashMap<NodeId, VariableAllocation>,
    pub closed_over_variables: HashSet<NodeId>,
    pub function_upvalues: HashMap<NodeId, Vec<UpvalueSource>>,
    scopes_stack: Vec<Scope>,
    pub errors: Vec<Exception>,
}

impl<'a> ResolveVariablesPhase<'a> {
    pub fn new(
        program: &'a mut Program,
        compilation_state: &'a mut CompilationState,
    ) -> ResolveVariablesPhase<'a> {
        ResolveVariablesPhase {
            program,
            compilation_state,
            exports: HashMap::new(),
            constants: HashMap::new(),
            variable_locations: HashMap::new(),
            variable_allocations: HashMap::new(),
            closed_over_variables: HashSet::new(),
            function_upvalues: HashMap::new(),
            scopes_stack: Vec::new(),
            errors: Vec::new(),
        }
    }

    pub fn run(&mut self) {
        self.compilation_state.ast().clone().walk(self);

        self.compilation_state
            .errors
            .append(&mut self.errors.clone());
    }

    fn push_child_scope(&mut self) {
        let parent = self.scopes_stack.last().unwrap();
        self.scopes_stack.push(Scope::new_child_scope(parent));
    }

    fn push_global_scope(&mut self, node: &Node) {
        self.scopes_stack.push(Scope::new_global_scope(node.id));
    }

    fn push_function_scope(&mut self, node: &Node) {
        self.scopes_stack.push(Scope::new_function_scope(node.id));
    }

    fn pop_scope(&mut self) -> Option<Scope> {
        self.scopes_stack.pop()
    }

    fn is_global_scope(&self) -> bool {
        self.scopes_stack.len() <= 1
    }

    fn define(&mut self, name: &str, node: &Node) -> Result<VariableLocation, ()> {
        let scope = self.scopes_stack.last_mut().unwrap();

        if scope.is_defined(name) {
            self.errors.push(Exception {
                source: Some(node.source.clone()),
                message: format!("{} already defined", name),
            });
            Err(())
        } else {
            scope.define(name, node.id);
            let index = scope.get_index(name).unwrap();

            let location = if self.is_global_scope() {
                VariableLocation::Global {
                    module: self.compilation_state.module_name,
                    name: name.into(),
                }
            } else {
                VariableLocation::Local { name: name.into() }
            };

            self.variable_locations.insert(node.id, location.clone());

            let allocation = if self.is_global_scope() {
                VariableAllocation::Global {
                    module: self.compilation_state.module_name,
                    name: name.into(),
                }
            } else {
                VariableAllocation::Local { index }
            };
            self.variable_allocations.insert(node.id, allocation);

            Ok(location)
        }
    }

    fn define_constant(&mut self, name: &str, node: &Node, value: Value) {
        if !self.is_global_scope() {
            unreachable!("constants only supported at global scope")
        }

        if self.define(name, node).is_ok() {
            self.constants.insert(name.to_owned(), value);
        }
    }

    fn lookup(&mut self, name: &str) -> Option<(VariableLocation, VariableAllocation)> {
        self.scopes_stack
            .clone()
            .iter()
            .rev()
            .enumerate()
            .find_map(|(i, s)| {
                if let Some(node_id) = s.get_node_id(name) {
                    if i >= self.scopes_stack.len() - 1 {
                        let location = VariableLocation::Global {
                            module: self.compilation_state.module_name,
                            name: name.into(),
                        };
                        let allocation = VariableAllocation::Global {
                            module: self.compilation_state.module_name,
                            name: name.into(),
                        };
                        Some((location, allocation))
                    } else if i == 0 {
                        let location = VariableLocation::Local { name: name.into() };
                        let allocation = VariableAllocation::Local {
                            index: s.get_index(name).unwrap(),
                        };
                        Some((location, allocation))
                    } else {
                        let location = VariableLocation::Closure {
                            name: name.into(),
                            nth_parent: i,
                        };

                        let mut function_depth = 0;
                        let mut current_func = self.scopes_stack.last().unwrap().get_function();
                        for s in self.scopes_stack.iter().rev().take(i + 1) {
                            if s.get_function() != current_func {
                                function_depth += 1;
                                current_func = s.get_function();
                            }
                        }

                        let allocation = if function_depth == 0 {
                            VariableAllocation::Local {
                                index: s.get_index(name).unwrap(),
                            }
                        } else if function_depth > 1 {
                            todo!()
                        } else {
                            // TODO: Handle variables multiple levels deep

                            let source = UpvalueSource::Local {
                                stack_index_relative_to_base: s.get_index(name).unwrap(),
                            };

                            let upvalue_index = self.get_upvalue(node_id, &source);

                            VariableAllocation::Upvalue {
                                source,
                                upvalue_index,
                            }
                        };

                        if function_depth > 0 {
                            self.closed_over_variables.insert(node_id);
                        }

                        Some((location, allocation))
                    }
                } else {
                    None
                }
            })
            .or_else(|| self.lookup_imported(name))
    }

    fn get_upvalue(&mut self, node_id: NodeId, source: &UpvalueSource) -> usize {
        // TODO: Handle variables multiple levels deep

        let function = self.scopes_stack.last().unwrap().get_function();
        self.scopes_stack
            .iter_mut()
            .find(|s| s.is_function_or_global() && s.get_function() == function)
            .unwrap()
            .get_or_allocate_upvalue(node_id, source)
    }

    fn lookup_imported(&self, name: &str) -> Option<(VariableLocation, VariableAllocation)> {
        for resolved_import in &self.compilation_state.imports {
            match &resolved_import.kind {
                super::ResolvedImportKind::Module => {
                    if resolved_import.short_name().unwrap_or("") == name {
                        unreachable!("should've been defined as a constant")
                    }
                }
                super::ResolvedImportKind::Field(field_name) => {
                    if name == field_name {
                        match self
                            .program
                            .get_module(&resolved_import.module_name)
                            .unwrap()
                            .exports
                            .get(name)
                        {
                            None => panic!("importing by field name, but field does not exist"),
                            Some(_) => {
                                let location = VariableLocation::Global {
                                    module: resolved_import.module_name,
                                    name: name.into(),
                                };
                                let allocation = VariableAllocation::Global {
                                    module: resolved_import.module_name,
                                    name: name.into(),
                                };
                                return Some((location, allocation));
                            }
                        }
                    }
                }
                super::ResolvedImportKind::AllFields => {
                    let module = self
                        .program
                        .get_module(&resolved_import.module_name)
                        .unwrap();
                    if module.exports.contains_key(name) {
                        let location = VariableLocation::Global {
                            module: resolved_import.module_name,
                            name: name.into(),
                        };
                        let allocation = VariableAllocation::Global {
                            module: resolved_import.module_name,
                            name: name.into(),
                        };
                        return Some((location, allocation));
                    }
                }
            }
        }

        None
    }

    fn resolve_variable(&mut self, node: &Node, name: &str) {
        match self.lookup(name) {
            Some((location, allocaction)) => {
                self.variable_locations.insert(node.id, location);
                self.variable_allocations.insert(node.id, allocaction);

                if self.constants.contains_key(name) {
                    let symbol = self.constants.get(name).unwrap().unwrap_symbol();
                    let module_name = ModuleName(symbol);
                    self.compilation_state
                        .tags
                        .insert(node.id, ModuleReference { module_name });
                }
            }
            None => self.errors.push(Exception {
                source: Some(node.source.clone()),
                message: format!("cannot find value '{}' in this scope", name),
            }),
        }
    }

    fn hoist_variables(&mut self, source_file_children: &[Node]) {
        for node in source_file_children {
            match &node.kind {
                NodeKind::Builtin { identifier } => {
                    let res = self.define(identifier.unwrap_identifier(), node);
                    if let Ok(location) = res {
                        self.exports
                            .insert(identifier.unwrap_identifier().to_owned(), location);
                    }
                }
                NodeKind::Var { pattern, .. } | NodeKind::Let { pattern, .. } => {
                    for node in pattern.iter() {
                        if let NodeKind::PatternIdentifier { identifier } = &node.kind {
                            let res = self.define(identifier.unwrap_identifier(), node);
                            if let Ok(location) = res {
                                self.exports
                                    .insert(identifier.unwrap_identifier().to_owned(), location);
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }

    fn define_all_in_pattern(&mut self, pattern: &Node) {
        for node in pattern.iter() {
            if let NodeKind::PatternIdentifier { identifier } = &node.kind {
                let _ = self.define(identifier.unwrap_identifier(), node);
            }
        }
    }

    fn resolve_all_in_pattern(&mut self, pattern: &Node) {
        for node in pattern.iter() {
            if let NodeKind::PatternIdentifier { identifier } = &node.kind {
                self.resolve_variable(node, identifier.unwrap_identifier())
            }
        }
    }
}

impl<'a> AstWalker for ResolveVariablesPhase<'a> {
    fn enter_node(&mut self, node: &Node) {
        match &node.kind {
            NodeKind::SourceFile(children) => {
                self.push_global_scope(node);
                self.hoist_variables(children);
            }
            NodeKind::Module(name) => {
                // TODO: somewhere - check if this AST matches the module name
                // the compiler is expecting
                let short_name = name.split('/').last().unwrap();
                let value = Value::Symbol(self.compilation_state.module_name.0);
                self.define_constant(short_name, node, value);
            }
            NodeKind::Import(ImportKind::Module { module_name }) => {
                let value = Value::Symbol(Ustr::from(module_name));
                // TODO: Consolidate short_name logic
                let short_name = module_name.split('/').last().unwrap();
                self.define_constant(short_name, node, value);
            }
            NodeKind::Body(_) => {
                self.push_child_scope();
            }
            NodeKind::FunctionLiteral { arg_names, .. } => {
                self.push_function_scope(node);
                for arg_name in arg_names {
                    let _ = self.define(arg_name.unwrap_identifier(), arg_name);
                }
            }
            NodeKind::For {
                pattern,
                enumerable: _,
                body: _,
            } => {
                self.push_child_scope();

                let iter_index = self
                    .scopes_stack
                    .last_mut()
                    .unwrap()
                    .allocate_anonymous_local();
                self.variable_allocations
                    .insert(node.id, VariableAllocation::Local { index: iter_index });

                let is_complex_pattern =
                    !matches!(&pattern.kind, NodeKind::PatternIdentifier { .. });

                if is_complex_pattern {
                    // need to allocate a second anonymous local to hold the iterator value
                    self.scopes_stack
                        .last_mut()
                        .unwrap()
                        .allocate_anonymous_local();
                }

                self.define_all_in_pattern(pattern);
            }
            NodeKind::Let { pattern, .. } | NodeKind::Var { pattern, .. } => {
                if self.is_global_scope() {
                    // already defined by hoist_variables(..)
                    return;
                }

                self.define_all_in_pattern(pattern);
            }
            NodeKind::VariableRef { identifier } => {
                self.resolve_variable(node, identifier.unwrap_identifier())
            }
            NodeKind::Assignment { pattern, .. } => self.resolve_all_in_pattern(pattern),
            // NodeKind::FieldAssignment handled by inner VariableRef
            _ => {}
        }
    }

    fn exit_node(&mut self, node: &Node) {
        match &node.kind {
            NodeKind::SourceFile(_) | NodeKind::FunctionLiteral { .. } => {
                let scope = self.pop_scope().unwrap();
                self.function_upvalues
                    .insert(node.id, scope.get_upvalue_sources());
            }
            NodeKind::Body(_) | NodeKind::For { .. } => {
                self.pop_scope().unwrap();
            }
            _ => {}
        }
    }
}
