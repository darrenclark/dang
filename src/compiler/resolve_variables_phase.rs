use std::collections::{HashMap, HashSet};

use crate::{
    ast::{AstWalker, Node, NodeId, NodeKind},
    interpreter::{exception, Exception},
    program::Program,
};

use super::{CompilationState, Compiler};

pub fn resolve_variables_phase(
    _compiler: &Compiler,
    compilation_state: &mut CompilationState,
    program: &mut Program,
) -> Result<(), Exception> {
    let mut phase = ResolveVariablesPhase::new(program, compilation_state);
    phase.run();
    if !phase.errors.is_empty() {
        compilation_state.errors.append(&mut phase.errors.clone());
        exception!("unable to resolve all variables")
    }
    compilation_state.exports = phase.exports;
    Ok(())
}

#[derive(Default, Debug)]
struct Scope {
    definitions: HashMap<String, NodeId>,
    definition_indices: Vec<String>,
}

#[derive(Debug)]
pub struct ResolveVariablesPhase<'a> {
    program: &'a mut Program,
    compilation_state: &'a CompilationState,
    // Assumes .* imports currently.  TODO: Support other import styles.
    pub definitions_to_usages: HashMap<NodeId, HashSet<NodeId>>,
    pub usages_to_definition: HashMap<NodeId, NodeId>,
    pub exports: HashMap<String, NodeId>,
    scopes: Vec<Scope>,
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
            definitions_to_usages: HashMap::new(),
            usages_to_definition: HashMap::new(),
            exports: HashMap::new(),
            scopes: Vec::new(),
            errors: Vec::new(),
        }
    }

    pub fn run(&mut self) {
        self.compilation_state.ast().walk(self)
    }

    fn push_scope(&mut self) {
        self.scopes.push(Scope::default());
    }

    fn pop_scope(&mut self) -> Option<Scope> {
        self.scopes.pop()
    }

    fn is_global_scope(&self) -> bool {
        self.scopes.len() <= 1
    }

    fn define(&mut self, name: &str, node: &Node) {
        let scope = self.scopes.last_mut().unwrap();

        if scope.definitions.contains_key(name) {
            self.errors.push(Exception {
                source: Some(node.source.clone()),
                message: format!("{} already defined", name),
            });
        } else {
            scope.definitions.insert(name.to_owned(), node.id);
            scope.definition_indices.push(name.to_owned());
        }
    }

    fn lookup(&self, name: &str) -> Option<NodeId> {
        self.scopes
            .iter()
            .rev()
            .find_map(|s| s.definitions.get(name).copied())
            .or_else(|| self.lookup_imported(name))
    }

    fn lookup_imported(&self, name: &str) -> Option<NodeId> {
        for resolved_import in &self.compilation_state.imports {
            match &resolved_import.kind {
                super::ResolvedImportKind::Module => panic!("module imports not supported yet"),
                super::ResolvedImportKind::Field(field_name) => {
                    if name == field_name {
                        match self
                            .program
                            .modules
                            .get_by_id(resolved_import.module_id)
                            .exports
                            .get(name)
                        {
                            None => panic!("importing by field name, but field does not exist"),
                            Some(node_id) => return Some(*node_id),
                        }
                    }
                }
                super::ResolvedImportKind::AllFields => {
                    let module = self.program.modules.get_by_id(resolved_import.module_id);
                    if let Some(node_id) = module.exports.get(name) {
                        return Some(*node_id);
                    }
                }
            }
        }

        None
    }

    fn resolve_variable(&mut self, variable_ref_node: &Node, name: &str) {
        if let Some(definition_id) = self.lookup(name) {
            self.usages_to_definition
                .insert(variable_ref_node.id, definition_id);
            self.definitions_to_usages
                .entry(definition_id)
                .or_default()
                .insert(variable_ref_node.id);
        } else {
            self.errors.push(Exception {
                source: Some(variable_ref_node.source.clone()),
                message: format!("cannot find value '{}' in this scope", name),
            });
        }
    }

    fn scan_ahead_for_global_functions(&mut self, source_file_children: &[Node]) {
        for node in source_file_children {
            match &node.kind {
                NodeKind::Builtin { identifier } => {
                    self.define(identifier.unwrap_identifier(), node)
                }
                NodeKind::Let { identifier, expr } => {
                    if expr.is_function_literal() {
                        self.define(identifier.unwrap_identifier(), node)
                    }
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
                self.scan_ahead_for_global_functions(children);
            }
            NodeKind::Body(_) => {
                self.push_scope();
            }
            NodeKind::FunctionLiteral { arg_names, .. } => {
                self.push_scope();
                for arg_name in arg_names {
                    self.define(arg_name.unwrap_identifier(), arg_name);
                }
            }
            NodeKind::For {
                var_name,
                enumerable: _,
                body: _,
            } => {
                self.push_scope();
                self.define(var_name.unwrap_identifier(), node)
            }
            NodeKind::Let { identifier, expr } => {
                if self.is_global_scope() && expr.is_function_literal() {
                    // already defined by scan_ahead_for_global_functions(..)
                    return;
                }

                self.define(identifier.unwrap_identifier(), node)
            }
            NodeKind::Var { identifier, .. } => self.define(identifier.unwrap_identifier(), node),
            NodeKind::VariableRef { identifier } => {
                self.resolve_variable(node, identifier.unwrap_identifier())
            }
            _ => {}
        }
    }

    fn exit_node(&mut self, node: &Node) {
        match &node.kind {
            NodeKind::SourceFile(_) => {
                self.exports = self.pop_scope().unwrap().definitions;
            }
            NodeKind::Body(_) => {
                self.pop_scope();
            }
            NodeKind::FunctionLiteral { .. } => {
                self.pop_scope();
            }
            NodeKind::For { .. } => {
                self.pop_scope();
            }
            _ => {}
        }
    }
}
