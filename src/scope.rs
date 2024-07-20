use std::collections::HashMap;

use lazy_static::lazy_static;
use ustr::Ustr;

use crate::{ast::NodeId, module::ModuleName, value::Value};

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
}

#[derive(Debug, Clone)]
pub enum VariableLocation {
    Global { module: ModuleName, name: Ustr },
    Closure { name: Ustr, nth_parent: usize },
    Local { name: Ustr },
    Constant(Value),
}

lazy_static! {
    static ref LOCAL: Ustr = Ustr::from("");
}

impl VariableLocation {
    pub fn get_name(&self) -> Option<Ustr> {
        match self {
            VariableLocation::Global { module: _, name } => Some(*name),
            VariableLocation::Closure {
                name,
                nth_parent: _,
            } => Some(*name),
            VariableLocation::Local { name } => Some(*name),
            VariableLocation::Constant(_) => None,
        }
    }

    pub fn get_key(&self) -> Option<(Ustr, Ustr)> {
        match self {
            VariableLocation::Global { module, name } => Some((module.0, *name)),
            VariableLocation::Closure {
                name,
                nth_parent: _,
            } => Some((*LOCAL, *name)),
            VariableLocation::Local { name } => Some((*LOCAL, *name)),
            VariableLocation::Constant(_) => None,
        }
    }
}
