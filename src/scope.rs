use std::collections::HashMap;

use lazy_static::lazy_static;
use ustr::Ustr;

use crate::{ast::NodeId, module::ModuleName};

#[derive(Debug)]
pub struct Scope {
    definitions: HashMap<String, NodeId>,
    base_offset: usize,
    indices: Vec<String>,
}

impl Scope {
    pub fn new_global_scope() -> Self {
        Self {
            definitions: HashMap::new(),
            base_offset: 0,
            indices: Vec::new(),
        }
    }

    pub fn new_function_scope() -> Self {
        Self {
            definitions: HashMap::new(),
            base_offset: 0,
            indices: Vec::new(),
        }
    }

    pub fn new_child_scope(parent: &Scope) -> Self {
        Self {
            definitions: HashMap::new(),
            base_offset: parent.base_offset + parent.definitions.len(),
            indices: Vec::new(),
        }
    }

    pub fn define(&mut self, name: &str, node_id: NodeId) {
        assert!(!self.is_defined(name));

        self.definitions.insert(name.to_owned(), node_id);
        self.indices.push(name.to_owned());
    }

    pub fn is_defined(&self, name: &str) -> bool {
        self.definitions.contains_key(name)
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

    pub fn into_exports(&self, module: ModuleName) -> HashMap<String, VariableLocation> {
        let mut res = HashMap::new();
        for (name, _) in &self.definitions {
            res.insert(
                name.clone(),
                VariableLocation::Global {
                    module,
                    name: Ustr::from(name),
                },
            );
        }
        res
    }
}

#[derive(Debug, Clone)]
pub enum VariableLocation {
    Global { module: ModuleName, name: Ustr },
    Closure { name: Ustr, nth_parent: usize },
    Local { name: Ustr },
}

lazy_static! {
    static ref LOCAL: Ustr = Ustr::from("");
}

impl VariableLocation {
    pub fn get_name(&self) -> Ustr {
        match self {
            VariableLocation::Global { module: _, name } => *name,
            VariableLocation::Closure {
                name,
                nth_parent: _,
            } => *name,
            VariableLocation::Local { name } => *name,
        }
    }

    pub fn get_key(&self) -> (Ustr, Ustr) {
        match self {
            VariableLocation::Global { module, name } => (module.0, *name),
            VariableLocation::Closure {
                name,
                nth_parent: _,
            } => (*LOCAL, *name),
            VariableLocation::Local { name } => (*LOCAL, *name),
        }
    }
}

/// VM equivalent of VariableLocation
#[derive(Debug, Clone)]
pub enum VariableAllocation {
    Global {
        module: ModuleName,
        name: Ustr,
    },
    /// index relative to the frame base
    Local {
        index: usize,
    },
}
