use std::collections::HashMap;

use ustr::Ustr;

use crate::{
    ast::{AstWalker, ImportKind, Node, NodeId, NodeKind},
    interpreter::{exception, Exception},
    program::Program,
    scope::{Scope, VariableLocation},
    value::Value,
};

use super::{CompilationState, Compiler};

pub fn resolve_variables_phase(
    _compiler: &Compiler,
    compilation_state: &mut CompilationState,
    program: &mut Program,
) -> Result<(), Exception> {
    let variable_locations: HashMap<NodeId, VariableLocation>;
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
        exports = phase.exports;
        constants = phase.constants;
    }

    compilation_state
        .variable_locations
        .clone_from(&variable_locations);
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
            global_scope: Scope::default(),
            scopes_stack: Vec::new(),
            errors: Vec::new(),
        }
    }

    pub fn run(&mut self) {
        self.compilation_state.ast().walk(self)
    }

    fn push_scope(&mut self) {
        self.scopes_stack.push(Scope::default());
    }

    fn pop_scope(&mut self) -> Option<Scope> {
        self.scopes_stack.pop()
    }

    fn is_global_scope(&self) -> bool {
        self.scopes_stack.len() <= 1
    }

    fn define(&mut self, name: &str, node: &Node) -> Result<(), ()> {
        let scope = self.scopes_stack.last_mut().unwrap();

        if scope.is_defined(name) {
            self.errors.push(Exception {
                source: Some(node.source.clone()),
                message: format!("{} already defined", name),
            });
            Err(())
        } else {
            scope.define(name, node.id);

            if self.is_global_scope() {
                self.variable_locations.insert(
                    node.id,
                    VariableLocation::Global {
                        module: self.compilation_state.module_name,
                        name: name.into(),
                    },
                );
            } else {
                self.variable_locations
                    .insert(node.id, VariableLocation::Local { name: name.into() });
            }
            Ok(())
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

    fn lookup(&self, name: &str) -> Option<VariableLocation> {
        self.scopes_stack
            .iter()
            .rev()
            .enumerate()
            .find_map(|(i, s)| {
                if let Some(_) = s.get_node_id(name) {
                    if i >= self.scopes_stack.len() - 1 {
                        Some(VariableLocation::Global {
                            module: self.compilation_state.module_name,
                            name: name.into(),
                        })
                    } else if i == 0 {
                        Some(VariableLocation::Local { name: name.into() })
                    } else {
                        Some(VariableLocation::Closure {
                            name: name.into(),
                            nth_parent: i,
                        })
                    }
                } else {
                    None
                }
            })
            .or_else(|| self.lookup_imported(name))
    }

    fn lookup_imported(&self, name: &str) -> Option<VariableLocation> {
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
                                return Some(VariableLocation::Global {
                                    module: resolved_import.module_name,
                                    name: name.into(),
                                })
                            }
                        }
                    }
                }
                super::ResolvedImportKind::AllFields => {
                    let module = self
                        .program
                        .get_module(&resolved_import.module_name)
                        .unwrap();
                    if let Some(_) = module.exports.get(name) {
                        return Some(VariableLocation::Global {
                            module: resolved_import.module_name,
                            name: name.into(),
                        });
                    }
                }
            }
        }

        None
    }

    fn resolve_variable(&mut self, node: &Node, name: &str) {
        match self.lookup(name) {
            Some(location) => {
                self.variable_locations.insert(node.id, location);
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
                NodeKind::Builtin { identifier }
                | NodeKind::Var { identifier, .. }
                | NodeKind::Let { identifier, .. } => {
                    let _ = self.define(identifier.unwrap_identifier(), node);
                }
                _ => {}
            }
        }
    }
}

impl<'a> AstWalker for ResolveVariablesPhase<'a> {
    fn enter_node(&mut self, node: &Node) {
        match &node.kind {
            NodeKind::SourceFile(children) => {
                self.push_scope();
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
                self.push_scope();
            }
            NodeKind::FunctionLiteral { arg_names, .. } => {
                self.push_scope();
                for arg_name in arg_names {
                    let _ = self.define(arg_name.unwrap_identifier(), arg_name);
                }
            }
            NodeKind::For {
                var_name,
                enumerable: _,
                body: _,
            } => {
                self.push_scope();
                let _ = self.define(var_name.unwrap_identifier(), node);
            }
            NodeKind::Let { identifier, .. } | NodeKind::Var { identifier, .. } => {
                if self.is_global_scope() {
                    // already defined by hoist_variables(..)
                    return;
                }

                let _ = self.define(identifier.unwrap_identifier(), node);
            }
            NodeKind::VariableRef { identifier } => {
                self.resolve_variable(node, identifier.unwrap_identifier())
            }
            NodeKind::Assignment { identifier, .. } => {
                self.resolve_variable(node, identifier.unwrap_identifier())
            }
            _ => {}
        }
    }

    fn exit_node(&mut self, node: &Node) {
        match &node.kind {
            NodeKind::SourceFile(_) => {
                self.global_scope = self.pop_scope().unwrap();
                self.exports.clone_from(
                    &self
                        .global_scope
                        .into_exports(self.compilation_state.module_name),
                );
            }
            NodeKind::Body(_) | NodeKind::FunctionLiteral { .. } | NodeKind::For { .. } => {
                let scope = self.pop_scope().unwrap();
                self.scopes.insert(node.id, scope);
            }
            _ => {}
        }
    }
}
