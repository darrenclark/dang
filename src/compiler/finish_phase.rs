use std::collections::HashMap;

use crate::{interpreter::Exception, module::Module, program::Program};

use super::{CompilationState, Compiler, PhaseImpl};

pub struct FinishPhase<'a> {
    compiler: &'a Compiler,
    compilation_state: &'a mut CompilationState,
    program: &'a mut Program,
}

impl<'a> FinishPhase<'a> {
    pub fn new(
        compiler: &'a Compiler,
        compilation_state: &'a mut CompilationState,
        program: &'a mut Program,
    ) -> FinishPhase<'a> {
        FinishPhase {
            compiler,
            compilation_state,
            program,
        }
    }
}

impl<'a> PhaseImpl for FinishPhase<'a> {
    fn run(&mut self) -> Result<(), Exception> {
        let module = Module {
            name: self.compilation_state.module_name.to_owned(),
            ast: Box::new(self.compilation_state.ast().clone()),
            // TODO: Copy globals across
            globals: HashMap::default(),
        };
        let _id = self.program.modules.insert(module)?;
        Ok(())
    }
}
