use std::collections::HashMap;

use crate::{ast::NodeId, compiler::variable::UpvalueSource};

#[derive(Debug, Clone)]
pub struct Scope {
    kind: ScopeKind,
    function: NodeId,
    definitions: HashMap<String, NodeId>,
    base_offset: usize,
    indices: Vec<String>,
    upvalues: Vec<(NodeId, UpvalueSource)>,
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
enum ScopeKind {
    Global,
    Function,
    Child,
}

impl Scope {
    pub fn new_global_scope(node_id: NodeId) -> Self {
        Self {
            kind: ScopeKind::Global,
            function: node_id,
            definitions: HashMap::new(),
            base_offset: 0,
            indices: Vec::new(),
            upvalues: Vec::new(),
        }
    }

    pub fn new_function_scope(node_id: NodeId) -> Self {
        Self {
            kind: ScopeKind::Function,
            function: node_id,
            definitions: HashMap::new(),
            base_offset: 0,
            indices: Vec::new(),
            upvalues: Vec::new(),
        }
    }

    pub fn new_child_scope(parent: &Scope) -> Self {
        let base_offset = if parent.kind != ScopeKind::Global {
            parent.base_offset + parent.indices.len()
        } else {
            0
        };

        Self {
            kind: ScopeKind::Child,
            function: parent.function,
            definitions: HashMap::new(),
            base_offset,
            indices: Vec::new(),
            upvalues: Vec::new(),
        }
    }

    pub fn define(&mut self, name: &str, node_id: NodeId) {
        assert!(!self.is_defined(name));

        self.definitions.insert(name.to_owned(), node_id);
        self.indices.push(name.to_owned());
    }

    pub fn allocate_anonymous_local(&mut self) -> usize {
        self.indices.push(String::new());
        self.indices.len() - 1 + self.base_offset
    }

    pub fn is_defined(&self, name: &str) -> bool {
        self.definitions.contains_key(name)
    }

    pub fn get_function(&self) -> NodeId {
        self.function
    }

    pub fn get_index(&self, name: &str) -> Option<usize> {
        self.indices
            .iter()
            .position(|n| n == name)
            .map(|i| i + self.base_offset)
    }

    pub fn get_node_id(&self, name: &str) -> Option<NodeId> {
        self.definitions.get(name).cloned()
    }

    pub fn get_definitions(&self) -> &HashMap<String, NodeId> {
        &self.definitions
    }

    pub fn get_upvalue_sources(&self) -> Vec<UpvalueSource> {
        self.upvalues
            .iter()
            .map(|(_, source)| source.clone())
            .collect()
    }

    pub fn is_function_or_global(&self) -> bool {
        self.kind != ScopeKind::Child
    }

    pub fn get_or_allocate_upvalue(
        &mut self,
        definition_node_id: NodeId,
        source: &UpvalueSource,
    ) -> usize {
        self.upvalues
            .iter()
            .position(|(id, _)| *id == definition_node_id)
            .unwrap_or_else(|| {
                self.upvalues.push((definition_node_id, source.clone()));
                self.upvalues.len() - 1
            })
    }
}
