use crate::{
    interpreter::{exception, Exception},
    module::module_name_to_file_path,
    program::Program,
    stdlib::load_stdlib_file,
};

use super::{CompilationState, Compiler, Input, PhaseImpl};

pub struct ReadFilePhase<'a> {
    compiler: &'a Compiler,
    compilation_state: &'a mut CompilationState,
    program: &'a mut Program,
}

impl<'a> ReadFilePhase<'a> {
    pub fn new(
        compiler: &'a Compiler,
        compilation_state: &'a mut CompilationState,
        program: &'a mut Program,
    ) -> ReadFilePhase<'a> {
        ReadFilePhase {
            compiler,
            compilation_state,
            program,
        }
    }
}

impl<'a> PhaseImpl for ReadFilePhase<'a> {
    fn run(&mut self) -> Result<(), Exception> {
        let source_code: Option<String> = match &self.compilation_state.input {
            Input::ModuleName(name) => {
                load_stdlib_file(&module_name_to_file_path(name)).map(|sc| sc.to_owned())
            }
            Input::SourceCode { text, .. } => Some(text.clone()),
        };

        match source_code {
            Some(source_code) => {
                self.compilation_state.source_code = source_code;
                Ok(())
            }
            None => {
                exception!(
                    "unable to find module: {}",
                    self.compilation_state.module_name
                )
            }
        }
    }
}
