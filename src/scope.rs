use std::collections::HashMap;

use lazy_static::lazy_static;
use ustr::Ustr;

use crate::{ast::NodeId, module::ModuleName};

#[derive(Default, Debug)]
pub struct Scope {
    size: usize,
    definitions: HashMap<String, NodeId>,
    definition_indices: HashMap<String, usize>,
}

impl Scope {
    pub fn define(&mut self, name: &str, node_id: NodeId) {
        assert!(!self.is_defined(name));

        self.definitions.insert(name.to_owned(), node_id);
        self.definition_indices.insert(name.to_owned(), self.size);
        self.size += 1;
    }

    pub fn is_defined(&self, name: &str) -> bool {
        self.definitions.contains_key(name)
    }

    pub fn get_node_id(&self, name: &str) -> Option<NodeId> {
        self.definitions.get(name).cloned()
    }

    pub fn get_index(&self, name: &str) -> Option<usize> {
        self.definition_indices.get(name).cloned()
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
