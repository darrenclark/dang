use crate::{
    ast::NodeKind,
    dang_parser,
    interpreter::{exception, Exception},
    program::Program,
};

use super::{CompilationState, Compiler, PhaseImpl};

pub struct CompileDependenciesPhase<'a> {
    compiler: &'a Compiler,
    compilation_state: &'a mut CompilationState,
    program: &'a mut Program,
}

impl<'a> CompileDependenciesPhase<'a> {
    pub fn new(
        compiler: &'a Compiler,
        compilation_state: &'a mut CompilationState,
        program: &'a mut Program,
    ) -> CompileDependenciesPhase<'a> {
        CompileDependenciesPhase {
            compiler,
            compilation_state,
            program,
        }
    }
}

impl<'a> PhaseImpl for CompileDependenciesPhase<'a> {
    fn run(&mut self) -> Result<(), Exception> {
        let imported_modules: Vec<String> = self
            .compilation_state
            .ast()
            .iter()
            .filter_map(|n| match &n.kind {
                NodeKind::Import(import_kind) => Some(import_kind.module_name().to_owned()),
                _ => None,
            })
            .collect();

        for m in imported_modules {
            match self.compiler.compile(&m, &mut self.program) {
                Ok(()) => {}
                Err(reasons) => {
                    let mut reasons = reasons;
                    self.compilation_state.errors.append(&mut reasons)
                }
            }
        }

        Ok(())
    }
}
