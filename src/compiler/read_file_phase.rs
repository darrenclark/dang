use crate::{
    interpreter::{exception, Exception},
    module::module_name_to_file_path,
    program::Program,
    stdlib::load_stdlib_file,
};

use super::{CompilationState, Compiler, Input};

pub fn read_file_phase(
    _compiler: &Compiler,
    compilation_state: &mut CompilationState,
    _program: &mut Program,
) -> Result<(), Exception> {
    let source_code: Option<String> = match &compilation_state.input {
        Input::ModuleName(name) => {
            load_stdlib_file(&module_name_to_file_path(name)).map(|sc| sc.to_owned())
        }
        Input::SourceCode { text, .. } => Some(text.clone()),
    };

    match source_code {
        Some(source_code) => {
            compilation_state.source_code = source_code;
            Ok(())
        }
        None => {
            exception!("unable to find module: {}", compilation_state.module_name)
        }
    }
}
