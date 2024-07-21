use std::collections::{HashMap, HashSet};

use ustr::Ustr;

use crate::{module::ModuleName, value::Value};

#[derive(Debug, Clone)]
pub struct StructInfo {
    pub module_name: ModuleName,
    pub fields: Vec<Ustr>,
    pub required_fields: HashSet<Ustr>,
    pub default_values: HashMap<Ustr, Value>,
}

impl StructInfo {
    pub fn new(module_name: ModuleName) -> StructInfo {
        StructInfo {
            module_name,
            fields: Vec::new(),
            required_fields: HashSet::new(),
            default_values: HashMap::new(),
        }
    }
}
