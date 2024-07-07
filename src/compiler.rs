use crate::{
    ast::Node,
    dang_parser,
    interpreter::{exception, Exception},
    module::{module_name_to_file_path, Module, ModuleId, ModulesMap},
    stdlib::load_stdlib_file,
};

#[derive(Debug, Default)]
pub struct Compiler {}

#[derive(Debug)]
enum ModuleLocation {
    Loaded,
    StdLib { path: String },
}

impl Compiler {
    pub fn compile_module(
        &self,
        module_name: &str,
        modules: &mut ModulesMap,
    ) -> Result<Vec<ModuleId>, Exception> {
        let ast = match self.lookup(module_name, modules)? {
            ModuleLocation::Loaded => return Ok(vec![]),
            ModuleLocation::StdLib { path } => self.parse_stdlib(&path)?,
        };

        ast.iter().for_each(|n| println!("{:?}", n));

        let module = Module {
            name: module_name.to_owned(),
            ast: Box::new(ast),
        };
        let id = modules.insert(module)?;

        Ok(vec![id])
    }

    fn lookup(&self, name: &str, modules: &mut ModulesMap) -> Result<ModuleLocation, Exception> {
        if modules.get_id_by_name(name).is_some() {
            return Ok(ModuleLocation::Loaded);
        }

        let module_path = module_name_to_file_path(name);

        if load_stdlib_file(&module_path).is_some() {
            return Ok(ModuleLocation::StdLib { path: module_path });
        }

        exception!("unable to find module {}", name)
    }

    fn parse_stdlib(&self, path: &str) -> Result<Node, Exception> {
        match dang_parser::parse(load_stdlib_file(path).unwrap(), path) {
            Ok(node) => Ok(node),
            Err(err) => {
                exception!("Failed to load standard library module {}:\n{}", path, err)
            }
        }
    }
}
