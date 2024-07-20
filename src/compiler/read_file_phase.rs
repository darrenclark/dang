use std::{fs, path::PathBuf};

use crate::{
    interpreter::{exception, Exception},
    module::module_name_to_file_path,
    program::Program,
    stdlib::load_stdlib_file,
};

use super::{CompilationState, Compiler, Input};

pub fn read_file_phase(
    compiler: &Compiler,
    compilation_state: &mut CompilationState,
    _program: &mut Program,
) -> Result<(), Exception> {
    let source_code: Option<String> = match &compilation_state.input {
        Input::ModuleName(name) => {
            if name.starts_with("Std") {
                load_stdlib_file(&module_name_to_file_path(name)).map(|sc| sc.to_owned())
            } else {
                load_file(
                    &compiler.module_search_paths,
                    &module_name_to_file_path(name),
                )
            }
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

fn load_file(module_search_paths: &[PathBuf], relative_path: &str) -> Option<String> {
    module_search_paths.iter().find_map(|module_search_path| {
        let path = std::path::Path::join(module_search_path, relative_path);
        fs::read_to_string(path).ok()
    })
}
