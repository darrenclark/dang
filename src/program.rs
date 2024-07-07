use std::collections::HashMap;

use crate::module::{ModuleId, ModulesMap};

/// Represents compile time info about a program.
///
/// Leveraged by Interpreter at runtime.
///
/// An eventual VM or WASM compiler would also use it.
#[derive(Default, Debug)]
pub struct Program {
    pub modules: ModulesMap,
    /// Mapping globals exported from modules to an index (their position in the vector)
    pub globals: Vec<(ModuleId, String)>,
    pub globals_lookup: HashMap<(ModuleId, String), usize>,
}
