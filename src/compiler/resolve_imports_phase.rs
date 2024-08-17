use crate::{ast::NodeKind, exception::Exception, program::Program};

use super::{CompilationState, Compiler, ResolvedImport};

pub fn resolve_imports_phase(
    _compiler: &Compiler,
    compilation_state: &mut CompilationState,
    _program: &mut Program,
) -> Result<(), Exception> {
    compilation_state.imports = compilation_state
        .ast()
        .iter()
        .filter_map(|n| match &n.kind {
            NodeKind::Import(import_kind) => {
                // TODO: for single field imports, verify field actually exists
                Some(ResolvedImport::new(import_kind))
            }
            _ => None,
        })
        .collect();

    // default imports for non-Std modules
    /*if !compilation_state.is_std() {
        for import_kind in &compiler.implicit_imports {
            let module_id = program
                .modules
                .get_id_by_name(import_kind.module_name())
                .unwrap();
            compilation_state
                .imports
                .push(ResolvedImport::new(module_id, import_kind))
        }
    }*/

    Ok(())
}
