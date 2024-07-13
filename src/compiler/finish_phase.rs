use std::collections::HashMap;

use crate::{interpreter::Exception, module::Module, program::Program};

use super::{CompilationState, Compiler};

pub fn finish_phase(
    _compiler: &Compiler,
    compilation_state: &mut CompilationState,
    program: &mut Program,
) -> Result<(), Exception> {
    let module = Module {
        name: compilation_state.module_name.to_owned(),
        ast: Box::new(compilation_state.ast().clone()),
        exports: compilation_state.exports.clone(),
    };
    let _id = program.modules.insert(module)?;
    Ok(())
}
