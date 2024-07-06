use std::{collections::HashMap, rc::Rc};

use crate::{
    ast::Node,
    interpreter::{exception, Exception},
};

#[derive(Debug)]
pub struct Module {
    pub name: String,
    pub ast: Box<Node>,
}

#[derive(Debug, Default)]
pub struct ModulesMap {
    name_to_id: HashMap<String, ModuleId>,
    // ModuleId is an index in to this list
    modules: Vec<Rc<Module>>,
}

impl ModulesMap {
    pub fn get_by_id(&self, id: ModuleId) -> Rc<Module> {
        self.modules[id.0 as usize].clone()
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ModuleId(u32);
