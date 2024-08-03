use core::fmt;
use std::collections::HashMap;

use ustr::Ustr;

use crate::{
    ast::{Node, NodeId},
    scope::VariableLocation,
    struct_info::StructInfo,
    value::Value,
    vm::chunk::Chunk,
};

#[derive(Debug)]
pub struct Module {
    pub name: ModuleName,
    pub ast: Box<Node>,
    pub exports: HashMap<String, VariableLocation>,
    pub constants: HashMap<String, Value>,
    pub variable_locations: HashMap<NodeId, VariableLocation>,
    pub struct_info: Option<StructInfo>,
    pub chunk: Chunk,
}

pub fn module_name_to_file_path(name: &str) -> String {
    format!("{}.dang", name)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ModuleName(pub Ustr);

impl ModuleName {
    pub fn is_unspecified(&self) -> bool {
        self.0 == ""
    }

    pub fn is_std(&self) -> bool {
        self.0.starts_with("Std")
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub fn short_name(&self) -> &str {
        self.0.as_str().split('/').last().unwrap()
    }
}

impl From<&str> for ModuleName {
    fn from(value: &str) -> Self {
        ModuleName(Ustr::from(value))
    }
}

impl From<String> for ModuleName {
    fn from(value: String) -> Self {
        ModuleName(Ustr::from(&value))
    }
}

impl Default for ModuleName {
    fn default() -> Self {
        ModuleName("".into())
    }
}

impl fmt::Display for ModuleName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
