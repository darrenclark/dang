use crate::{ast::NodeKind, interpreter::Exception, program::Program};

use super::{CompilationState, Compiler};

pub fn compile_dependencies_phase(
    compiler: &Compiler,
    compilation_state: &mut CompilationState,
    program: &mut Program,
) -> Result<(), Exception> {
    let imported_modules: Vec<String> = compilation_state
        .ast()
        .iter()
        .filter_map(|n| match &n.kind {
            NodeKind::Import(import_kind) => Some(import_kind.module_name().to_owned()),
            _ => None,
        })
        .collect();

    for m in imported_modules {
        match compiler.compile(&m, program) {
            Ok(()) => {}
            Err(reasons) => {
                let mut reasons = reasons;
                compilation_state.errors.append(&mut reasons)
            }
        }
    }

    Ok(())
}
