use crate::{
    ast::NodeKind,
    exception::{exception, Exception},
    program::Program,
};

use super::{CompilationState, Compiler, Input};

pub fn compile_dependencies_phase(
    compiler: &Compiler,
    compilation_state: &mut CompilationState,
    program: &mut Program,
) -> Result<(), Exception> {
    let mut imported_modules: Vec<Input> = Vec::new();

    /*if !compilation_state.is_std() {
        for import in &compiler.implicit_imports {
            imported_modules.push(Input::ModuleName(import.module_name().to_owned()));
        }
    }*/

    imported_modules.extend(
        compilation_state
            .ast()
            .iter()
            .filter_map(|n| match &n.kind {
                NodeKind::Import(import_kind) => {
                    Some(Input::ModuleName(import_kind.module_name().to_owned()))
                }
                _ => None,
            }),
    );

    for m in imported_modules {
        match compiler.compile(m.clone(), program) {
            Ok(()) => {}
            Err(reasons) => {
                let mut reasons = reasons;
                compilation_state.errors.append(&mut reasons);
                exception!("Failed to load dependency: {}", m.module_name())
            }
        }
    }

    Ok(())
}
