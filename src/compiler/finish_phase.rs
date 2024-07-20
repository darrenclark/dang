use crate::{interpreter::Exception, module::Module, program::Program};

use super::{CompilationState, Compiler};

pub fn finish_phase(
    _compiler: &Compiler,
    compilation_state: &mut CompilationState,
    program: &mut Program,
) -> Result<(), Exception> {
    let module = Module {
        name: compilation_state.module_name,
        ast: Box::new(compilation_state.ast().clone()),
        exports: compilation_state.exports.clone(),
        variable_locations: compilation_state.variable_locations.clone(),
    };
    let _id = program.insert_module(compilation_state.module_name, module);
    Ok(())
}
