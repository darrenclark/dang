use crate::{ast::NodeKind, interpreter::Exception, program::Program};

use super::{CompilationState, Compiler, ResolvedImport};

const IMPLICIT_IMPORTS: [&str; 3] = ["Std/Assert", "Std/Builtins", "Std/Enum"];

pub fn resolve_imports_phase(
    _compiler: &Compiler,
    compilation_state: &mut CompilationState,
    program: &mut Program,
) -> Result<(), Exception> {
    compilation_state.imports = compilation_state
        .ast()
        .iter()
        .filter_map(|n| match &n.kind {
            NodeKind::Import(import_kind) => {
                let module_id = program
                    .modules
                    .get_id_by_name(import_kind.module_name())
                    .unwrap();
                // TODO: for single field imports, verify field actually exists
                Some(ResolvedImport::new(module_id, import_kind))
            }
            _ => None,
        })
        .collect();

    // default imports for non-Std modules
    if !compilation_state.module_name.starts_with("Std") {
        for module_name in IMPLICIT_IMPORTS {
            println!("{}", module_name);
            let module_id = program.modules.get_id_by_name(module_name).unwrap();
            compilation_state
                .imports
                .push(ResolvedImport::new_implicit(module_id, module_name))
        }
    }

    Ok(())
}
