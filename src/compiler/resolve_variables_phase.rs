use std::collections::HashMap;

use ustr::Ustr;

use crate::{
    ast::{AstWalker, ImportKind, Node, NodeId, NodeKind},
    interpreter::{exception, Exception},
    program::Program,
    scope::{Scope, VariableAllocation, VariableLocation},
    value::Value,
};

use super::{CompilationState, Compiler};

pub fn resolve_variables_phase(
    _compiler: &Compiler,
    compilation_state: &mut CompilationState,
    program: &mut Program,
) -> Result<(), Exception> {
    let variable_locations: HashMap<NodeId, VariableLocation>;
    let variable_allocations: HashMap<NodeId, VariableAllocation>;
    let exports: HashMap<String, VariableLocation>;
    let constants: HashMap<String, Value>;

    {
        let mut phase = ResolveVariablesPhase::new(program, compilation_state);
        phase.run();
        if !phase.errors.is_empty() {
            compilation_state.errors.append(&mut phase.errors.clone());
            exception!("unable to resolve all variables")
        }

        variable_locations = phase.variable_locations;
        variable_allocations = phase.variable_allocations;
        exports = phase.exports;
        constants = phase.constants;
    }

    compilation_state
        .variable_locations
        .clone_from(&variable_locations);
    compilation_state
        .variable_allocations
        .clone_from(&variable_allocations);
    compilation_state.exports.clone_from(&exports);
    compilation_state.constants.clone_from(&constants);
    Ok(())
}

#[derive(Debug)]
pub struct ResolveVariablesPhase<'a> {
    program: &'a mut Program,
    compilation_state: &'a CompilationState,
    pub exports: HashMap<String, VariableLocation>,
    pub constants: HashMap<String, Value>,
    pub scopes: HashMap<NodeId, Scope>,
    pub variable_locations: HashMap<NodeId, VariableLocation>,
    pub variable_allocations: HashMap<NodeId, VariableAllocation>,
    pub global_scope: Scope,
    scopes_stack: Vec<Scope>,
    pub errors: Vec<Exception>,
}

impl<'a> ResolveVariablesPhase<'a> {
    pub fn new(
        program: &'a mut Program,
        compilation_state: &'a CompilationState,
    ) -> ResolveVariablesPhase<'a> {
        ResolveVariablesPhase {
            program,
            compilation_state,
            exports: HashMap::new(),
            constants: HashMap::new(),
            scopes: HashMap::new(),
            variable_locations: HashMap::new(),
            variable_allocations: HashMap::new(),
            global_scope: Scope::new_global_scope(),
            scopes_stack: Vec::new(),
            errors: Vec::new(),
        }
    }

    pub fn run(&mut self) {
        self.compilation_state.ast().walk(self)
    }

    fn push_child_scope(&mut self) {
        let parent = self.scopes_stack.last().unwrap();
        self.scopes_stack.push(Scope::new_child_scope(parent));
    }

    fn push_global_scope(&mut self) {
        self.scopes_stack.push(Scope::new_global_scope());
    }

    fn push_function_scope(&mut self) {
        self.scopes_stack.push(Scope::new_function_scope());
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

    fn lookup(&self, name: &str) -> Option<(VariableLocation, VariableAllocation)> {
        self.scopes_stack
            .iter()
            .rev()
            .enumerate()
            .find_map(|(i, s)| {
                if s.get_node_id(name).is_some() {
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
                        // TODO: Handle closed over variables
                        let allocation = VariableAllocation::Local {
                            index: s.get_index(name).unwrap(),
                        };
                        Some((location, allocation))
                    }
                } else {
                    None
                }
            })
            .or_else(|| self.lookup_imported(name))
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
                self.push_global_scope();
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
                self.push_function_scope();
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
            NodeKind::SourceFile(_) => {
                self.global_scope = self.pop_scope().unwrap();
            }
            NodeKind::Body(_) | NodeKind::FunctionLiteral { .. } | NodeKind::For { .. } => {
                let scope = self.pop_scope().unwrap();
                self.scopes.insert(node.id, scope);
            }
            _ => {}
        }
    }
}
