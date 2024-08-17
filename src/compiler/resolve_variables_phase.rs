use std::collections::HashMap;

use ustr::Ustr;

use crate::{
    ast::{AstWalker, ImportKind, Node, NodeId, NodeKind},
    exception::{exception, Exception},
    module::ModuleName,
    program::Program,
    scope::Scope,
    value::Value,
};

use super::{
    variable::{UpvalueSource, Variable, VariableAllocation, VariableRef},
    CompilationState, Compiler,
};

#[derive(Debug)]
pub struct ModuleReference {
    pub module_name: ModuleName,
}

pub fn resolve_variables_phase(
    _compiler: &Compiler,
    compilation_state: &mut CompilationState,
    program: &mut Program,
) -> Result<(), Exception> {
    let exports: HashMap<String, Variable>;
    let constants: HashMap<String, Value>;

    {
        let mut phase = ResolveVariablesPhase::new(program, compilation_state);
        phase.run();
        if !phase.errors.is_empty() {
            exception!("unable to resolve all variables")
        }

        exports = phase.exports;
        constants = phase.constants;
    }

    compilation_state.exports.clone_from(&exports);
    compilation_state.constants.clone_from(&constants);
    Ok(())
}

#[derive(Debug)]
pub struct ResolveVariablesPhase<'a> {
    program: &'a mut Program,
    compilation_state: &'a mut CompilationState,
    pub exports: HashMap<String, Variable>,
    pub constants: HashMap<String, Value>,
    scopes_stack: Vec<NodeId>,
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

    fn push_child_scope(&mut self, node: &Node) {
        let parent = self.current_scope();

        let scope = Scope::new_child_scope(node.id, parent);
        self.compilation_state.tags.insert(node.id, scope);

        self.scopes_stack.push(node.id);
    }

    fn push_global_scope(&mut self, node: &Node) {
        let scope = Scope::new_global_scope(node.id);
        self.compilation_state.tags.insert(node.id, scope);

        self.scopes_stack.push(node.id);
    }

    fn push_function_scope(&mut self, node: &Node) {
        let parent = self.current_scope();
        let scope = Scope::new_function_scope(node.id, parent);
        self.compilation_state.tags.insert(node.id, scope);

        self.scopes_stack.push(node.id);
    }

    fn pop_scope(&mut self) -> Option<&Scope> {
        self.scopes_stack
            .pop()
            .map(|node_id| self.compilation_state.tags.get::<Scope>(node_id).unwrap())
    }

    fn is_global_scope(&self) -> bool {
        self.scopes_stack.len() <= 1
    }

    fn current_scope(&self) -> &Scope {
        let node_id = self.scopes_stack.last().unwrap();
        self.compilation_state.tags.get::<Scope>(*node_id).unwrap()
    }

    fn current_scope_mut(&mut self) -> &mut Scope {
        let node_id = self.scopes_stack.last().unwrap();
        self.compilation_state
            .tags
            .get_mut::<Scope>(*node_id)
            .unwrap()
    }

    fn define(&mut self, name: &str, node: &Node) -> Result<(), ()> {
        let scope = self.current_scope_mut();

        if scope.is_defined(name) {
            self.errors.push(Exception {
                source: Some(node.source.clone()),
                message: format!("{} already defined", name),
            });
            Err(())
        } else {
            scope.define(name, node.id);
            let index = scope.get_index(name).unwrap();

            let allocation = if self.is_global_scope() {
                VariableAllocation::Global {
                    module: self.compilation_state.module_name,
                    name: name.into(),
                }
            } else {
                VariableAllocation::Local { index }
            };
            self.compilation_state
                .tags
                .insert(node.id, VariableRef::new(allocation));

            self.compilation_state
                .tags
                .insert(node.id, Variable::default());

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

    fn lookup(&mut self, name: &str) -> Option<VariableRef> {
        self.lookup_in_scope(name, *self.scopes_stack.last().unwrap())
            .or_else(|| self.lookup_imported(name))
    }

    fn lookup_in_scope(&mut self, name: &str, scope_node: NodeId) -> Option<VariableRef> {
        let scope = self
            .compilation_state
            .tags
            .get::<Scope>(scope_node)
            .unwrap();

        if let Some(node_id) = scope.get_node_id(name) {
            let allocation = if scope.is_global() {
                VariableAllocation::Global {
                    module: self.compilation_state.module_name,
                    name: Ustr::from(name),
                }
            } else {
                VariableAllocation::Local {
                    index: scope.get_index(name).unwrap(),
                }
            };
            let mut variable_ref = VariableRef::new(allocation);
            variable_ref.definition_node_id = Some(node_id);
            Some(variable_ref)
        } else if let Some(parent_node) = scope.parent_node_id() {
            if let Some(mut variable_ref) = self.lookup_in_scope(name, parent_node) {
                if !variable_ref.allocation.is_global()
                    && self.different_function_scope(parent_node, scope_node)
                {
                    // TODO: get correct NodeId
                    let definition_node_id = variable_ref.definition_node_id.unwrap();

                    self.compilation_state
                        .tags
                        .get_mut::<Variable>(definition_node_id)
                        .unwrap()
                        .closed_over = true;

                    let upvalue_source = match variable_ref.allocation {
                        VariableAllocation::Local { index } => UpvalueSource::Local {
                            stack_index_relative_to_base: index,
                        },
                        VariableAllocation::Upvalue {
                            source: _,
                            upvalue_index,
                        } => UpvalueSource::Upvalue { upvalue_index },
                        _ => unreachable!(),
                    };

                    let upvalue_index = self
                        .compilation_state
                        .tags
                        // scope_node will always be a function scope here
                        .get_mut::<Scope>(scope_node)
                        .unwrap()
                        .get_or_allocate_upvalue(definition_node_id, &upvalue_source);

                    variable_ref.allocation = VariableAllocation::Upvalue {
                        source: upvalue_source,
                        upvalue_index,
                    };

                    Some(variable_ref)
                } else {
                    Some(variable_ref)
                }
            } else {
                None
            }
        } else {
            None
        }
    }

    fn different_function_scope(&self, parent_scope_node: NodeId, scope_node: NodeId) -> bool {
        let scope = self
            .compilation_state
            .tags
            .get::<Scope>(scope_node)
            .unwrap();

        let parent_scope = self
            .compilation_state
            .tags
            .get::<Scope>(parent_scope_node)
            .unwrap();

        scope.get_function() != parent_scope.get_function()
    }

    fn lookup_imported(&self, name: &str) -> Option<VariableRef> {
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
                                let allocation = VariableAllocation::Global {
                                    module: resolved_import.module_name,
                                    name: name.into(),
                                };
                                return Some(VariableRef::new(allocation));
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
                        let allocation = VariableAllocation::Global {
                            module: resolved_import.module_name,
                            name: name.into(),
                        };
                        return Some(VariableRef::new(allocation));
                    }
                }
            }
        }

        None
    }

    fn resolve_variable(&mut self, node: &Node, name: &str) {
        match self.lookup(name) {
            Some(variable_ref) => {
                self.compilation_state.tags.insert(node.id, variable_ref);

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
                    if let Ok(()) = res {
                        self.exports.insert(
                            identifier.unwrap_identifier().to_owned(),
                            Variable::default(),
                        );
                    }
                }
                NodeKind::Var { pattern, .. } | NodeKind::Let { pattern, .. } => {
                    for node in pattern.iter() {
                        if let NodeKind::PatternIdentifier { identifier } = &node.kind {
                            let res = self.define(identifier.unwrap_identifier(), node);
                            if let Ok(()) = res {
                                self.exports.insert(
                                    identifier.unwrap_identifier().to_owned(),
                                    Variable::default(),
                                );
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
                self.push_child_scope(node);
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
                self.push_child_scope(node);

                let iter_index = self.current_scope_mut().allocate_anonymous_local();
                self.compilation_state.tags.insert(
                    node.id,
                    VariableRef::new(VariableAllocation::Local { index: iter_index }),
                );

                let is_complex_pattern =
                    !matches!(&pattern.kind, NodeKind::PatternIdentifier { .. });

                if is_complex_pattern {
                    // need to allocate a second anonymous local to hold the iterator value
                    self.current_scope_mut().allocate_anonymous_local();
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
                self.pop_scope().unwrap();
            }
            NodeKind::Body(_) | NodeKind::For { .. } => {
                self.pop_scope().unwrap();
            }
            NodeKind::Let { pattern, .. } | NodeKind::Var { pattern, .. } => {
                // Handle Let and Var on the exit, so that the variables are not visible to the
                // expression on the right side of the assignment

                if self.is_global_scope() {
                    // already defined by hoist_variables(..)
                    return;
                }

                self.define_all_in_pattern(pattern);
            }
            _ => {}
        }
    }
}
