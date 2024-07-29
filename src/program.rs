use std::{collections::HashMap, rc::Rc};

use crate::{
    ast::Node,
    module::{Module, ModuleName},
    scope::VariableLocation,
};

/// Represents compile time info about a program.
///
/// Leveraged by Interpreter at runtime.
///
/// An eventual VM or WASM compiler would also use it.
#[derive(Default, Debug)]
pub struct Program {
    modules: Vec<(ModuleName, Rc<Module>)>,
    /// Mapping globals exported from modules to an index (their position in the vector)
    pub globals: Vec<(ModuleName, String)>,
    pub globals_lookup: HashMap<(ModuleName, String), usize>,
}

impl Program {
    pub fn module_names(&self) -> Vec<ModuleName> {
        self.modules.iter().map(|(n, _)| *n).collect()
    }

    pub fn get_module(&self, module_name: &ModuleName) -> Option<Rc<Module>> {
        self.modules
            .iter()
            .find(|(name, _)| name == module_name)
            .map(|(_, m)| m)
            .cloned()
    }

    pub fn insert_module(&mut self, name: ModuleName, module: Module) {
        if self.get_module(&name).is_some() {
            panic!("Attempted to insert a duplicate module: {:?}", name)
        }

        self.modules.push((name, Rc::new(module)));
    }

    pub fn variable_location(&self, node: &Node) -> VariableLocation {
        match self
            .get_module(&node.id.module())
            .expect("expected module loaded")
            .variable_locations
            .get(&node.id)
        {
            Some(l) => l.clone(),
            None => panic!(
                "variable location not found for node at {}: {:?}",
                node.source, node
            ),
        }
    }
}
