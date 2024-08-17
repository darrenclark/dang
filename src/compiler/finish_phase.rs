use crate::{exception::Exception, module::Module, program::Program};

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
        constants: compilation_state.constants.clone(),
        struct_info: compilation_state.struct_info.clone(),
        function: compilation_state.function.clone(),
    };
    program.insert_module(compilation_state.module_name, module);
    Ok(())
}
