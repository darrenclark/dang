use crate::{
    dang_parser,
    interpreter::{exception, Exception},
    program::Program,
};

use super::{CompilationState, Compiler, PhaseImpl};

pub struct ParseAstPhase<'a> {
    compiler: &'a Compiler,
    compilation_state: &'a mut CompilationState,
    program: &'a mut Program,
}

impl<'a> ParseAstPhase<'a> {
    pub fn new(
        compiler: &'a Compiler,
        compilation_state: &'a mut CompilationState,
        program: &'a mut Program,
    ) -> ParseAstPhase<'a> {
        ParseAstPhase {
            compiler,
            compilation_state,
            program,
        }
    }
}

impl<'a> PhaseImpl for ParseAstPhase<'a> {
    fn run(&mut self) -> Result<(), Exception> {
        match dang_parser::parse(
            &self.compilation_state.source_code,
            &self.compilation_state.module_name,
        ) {
            Ok(node) => {
                self.compilation_state.ast = Some(node);
                Ok(())
            }
            Err(err) => {
                exception!(
                    "Failed to load standard library module {}:\n{}",
                    self.compilation_state.module_name,
                    err
                )
            }
        }
    }
}
