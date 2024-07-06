use crate::{
    ast::Node,
    interpreter::Exception,
    module::{Module, ModuleId, ModulesMap},
};

#[derive(Debug, Default)]
pub struct Compiler {}

impl Compiler {
    pub fn compile(
        &self,
        name: String,
        ast: Node,
        modules: &mut ModulesMap,
    ) -> Result<Vec<ModuleId>, Exception> {
        let module = Module {
            name,
            ast: Box::new(ast),
        };
        let id = modules.insert(module)?;

        Ok(vec![id])
    }
}
