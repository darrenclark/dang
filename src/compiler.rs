pub mod compile_dependencies_phase;
pub mod finish_phase;
pub mod node_ids_pass;
pub mod parse_ast_phase;
pub mod read_file_phase;
pub mod resolve_variables;

use std::collections::HashMap;

use compile_dependencies_phase::CompileDependenciesPhase;
use finish_phase::FinishPhase;
use node_ids_pass::NodeIdsPass;
use parse_ast_phase::ParseAstPhase;
use read_file_phase::ReadFilePhase;

use crate::{
    ast::{Node, NodeKind},
    dang_parser,
    interpreter::{exception, Exception},
    module::{module_name_to_file_path, Module, ModuleId},
    program::Program,
    stdlib::load_stdlib_file,
};

#[derive(Debug)]
pub enum Input {
    ModuleName(String),
    SourceCode { text: String, name: String },
}

#[derive(Debug)]
pub struct CompilationState {
    pub input: Input,
    pub module_name: String,
    pub errors: Vec<Exception>,
    pub source_code: String,
    pub ast: Option<Node>,
}

impl CompilationState {
    fn new(input: Input) -> CompilationState {
        let module_name = match &input {
            Input::ModuleName(n) => n.to_owned(),
            Input::SourceCode { text: _, name } => name.to_owned(),
        };

        CompilationState {
            input,
            module_name,
            errors: Vec::new(),
            source_code: String::new(),
            ast: None,
        }
    }

    pub fn ast(&self) -> &Node {
        self.ast.as_ref().unwrap()
    }
}

trait PhaseImpl {
    fn run(&mut self) -> Result<(), Exception>;
}

enum Phase {
    ReadFile,
    ParseAst,
    CompileDependencies,
    Finish,
}

impl Phase {
    const PHASES: [Self; 4] = [
        Self::ReadFile,
        Self::ParseAst,
        Self::CompileDependencies,
        Self::Finish,
    ];

    fn new_impl<'a>(
        &self,
        compiler: &'a Compiler,
        compilation_state: &'a mut CompilationState,
        program: &'a mut Program,
    ) -> Box<dyn PhaseImpl + 'a> {
        match self {
            Phase::ReadFile => Box::new(ReadFilePhase::new(compiler, compilation_state, program)),
            Phase::ParseAst => Box::new(ParseAstPhase::new(compiler, compilation_state, program)),
            Phase::CompileDependencies => Box::new(CompileDependenciesPhase::new(
                compiler,
                compilation_state,
                program,
            )),
            Phase::Finish => Box::new(FinishPhase::new(compiler, compilation_state, program)),
        }
    }
}

#[derive(Debug, Default)]
pub struct Compiler {}

#[derive(Debug)]
enum ModuleLocation {
    Loaded,
    StdLib { path: String },
}

impl Compiler {
    pub fn compile(&self, module_name: &str, program: &mut Program) -> Result<(), Vec<Exception>> {
        if program.modules.get_by_name(module_name).is_some() {
            return Ok(());
        }

        let mut state = CompilationState::new(Input::ModuleName(module_name.to_owned()));

        for phase in Phase::PHASES {
            let result = { phase.new_impl(&self, &mut state, program).run() };
            match result {
                Ok(_) => {}
                Err(exception) => {
                    let mut errors = state.errors.clone();
                    errors.push(exception);
                    return Err(errors);
                }
            }
        }

        Ok(())
    }
}
