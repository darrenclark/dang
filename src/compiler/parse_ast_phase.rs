use crate::{
    dang_parser,
    exception::{exception, Exception},
    program::Program,
};

use super::{CompilationState, Compiler};

pub fn parse_ast_phase(
    _compiler: &Compiler,
    compilation_state: &mut CompilationState,
    _program: &mut Program,
) -> Result<(), Exception> {
    match dang_parser::parse(
        &compilation_state.source_code,
        compilation_state.module_name.0.as_str(),
    ) {
        Ok(node) => {
            compilation_state.ast = Some(node);
            Ok(())
        }
        Err(err) => {
            exception!(
                "Failed to load standard library module {}:\n{}",
                compilation_state.module_name,
                err
            )
        }
    }
}
