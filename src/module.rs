use std::{collections::HashMap, rc::Rc};

use crate::{
    ast::{Node, NodeId},
    interpreter::{exception, Exception},
};

#[derive(Debug)]
pub struct Module {
    pub name: String,
    pub ast: Box<Node>,
    pub globals: HashMap<String, NodeId>,
}

#[derive(Debug, Default)]
pub struct ModulesMap {
    name_to_id: HashMap<String, ModuleId>,
    // ModuleId is an index in to this list
    pub modules: Vec<Rc<Module>>,
}

pub fn module_name_to_file_path(name: &str) -> String {
    format!("{}.dang", name)
}

impl ModulesMap {
    pub fn module_ids(&self) -> Vec<ModuleId> {
        self.modules
            .iter()
            .enumerate()
            .map(|(i, _)| ModuleId(i as u32))
            .collect()
    }

    pub fn get_by_id(&self, id: ModuleId) -> Rc<Module> {
        self.modules[id.0 as usize].clone()
    }

    pub fn get_id_by_name(&self, name: &str) -> Option<ModuleId> {
        self.name_to_id.get(name).cloned()
    }

    pub fn get_by_name(&self, name: &str) -> Option<Rc<Module>> {
        self.get_id_by_name(name).map(|id| self.get_by_id(id))
    }

    pub fn next_id(&self) -> ModuleId {
        ModuleId(self.modules.len() as u32)
    }

    pub fn insert(&mut self, module: Module) -> Result<ModuleId, Exception> {
        let name = module.name.clone();

        if self.name_to_id.contains_key(&name) {
            exception!("module {} already loaded", &module.name)
        }

        let id = ModuleId(self.modules.len() as u32);
        self.modules.push(Rc::new(module));
        self.name_to_id.insert(name.clone(), id);
        Ok(id)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ModuleId(u32);

impl ModuleId {
    pub fn from_raw(id: u32) -> ModuleId {
        ModuleId(id)
    }

    pub fn raw_id(&self) -> u32 {
        self.0
    }
}

pub const UNSPECIFIED_MODULE_ID: u32 = u32::MAX;

impl Default for ModuleId {
    fn default() -> Self {
        ModuleId(UNSPECIFIED_MODULE_ID)
    }
}
