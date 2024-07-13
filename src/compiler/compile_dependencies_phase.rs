use crate::{
    ast::NodeKind,
    interpreter::{exception, Exception},
    program::Program,
};

use super::{CompilationState, Compiler, Input};

pub fn compile_dependencies_phase(
    compiler: &Compiler,
    compilation_state: &mut CompilationState,
    program: &mut Program,
) -> Result<(), Exception> {
    if !compilation_state.module_name.starts_with("Std") {
        match compiler.compile(Input::ModuleName("Std".to_owned()), program) {
            Ok(()) => {}
            Err(reasons) => {
                let mut reasons = reasons;
                compilation_state.errors.append(&mut reasons);
                exception!("Failed to load standard lib")
            }
        }
    }

    let imported_modules: Vec<Input> = compilation_state
        .ast()
        .iter()
        .filter_map(|n| match &n.kind {
            NodeKind::Import(import_kind) => {
                Some(Input::ModuleName(import_kind.module_name().to_owned()))
            }
            _ => None,
        })
        .collect();

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
