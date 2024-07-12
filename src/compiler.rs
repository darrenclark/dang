pub mod compile_dependencies_phase;
pub mod finish_phase;
pub mod node_ids_phase;
pub mod parse_ast_phase;
pub mod read_file_phase;
pub mod resolve_variables;

use compile_dependencies_phase::compile_dependencies_phase;
use finish_phase::finish_phase;
use node_ids_phase::node_ids_phase;
use parse_ast_phase::parse_ast_phase;
use read_file_phase::read_file_phase;

use crate::{ast::Node, interpreter::Exception, module::ModuleId, program::Program};

#[derive(Debug)]
pub enum Input {
    ModuleName(String),
    SourceCode { text: String, name: String },
}

#[derive(Debug)]
pub struct CompilationState {
    pub input: Input,
    pub module_name: String,
    pub module_id: ModuleId,
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
            module_id: ModuleId::default(),
            errors: Vec::new(),
            source_code: String::new(),
            ast: None,
        }
    }

    pub fn ast(&self) -> &Node {
        self.ast.as_ref().unwrap()
    }

    pub fn ast_mut(&mut self) -> &mut Node {
        self.ast.as_mut().unwrap()
    }
}

type PhaseFn = fn(&Compiler, &mut CompilationState, &mut Program) -> Result<(), Exception>;

#[derive(PartialEq, Eq)]
pub enum Phase {
    ReadFile,
    ParseAst,
    CompileDependencies,
    NodeIds,
    Finish,
}

impl Phase {
    const PHASES: [(Self, PhaseFn); 5] = [
        (Self::ReadFile, read_file_phase),
        (Self::ParseAst, parse_ast_phase),
        (Self::CompileDependencies, compile_dependencies_phase),
        (Self::NodeIds, node_ids_phase),
        (Self::Finish, finish_phase),
    ];
}

#[derive(Debug, Default)]
pub struct Compiler {}

impl Compiler {
    pub fn compile(&self, module_name: &str, program: &mut Program) -> Result<(), Vec<Exception>> {
        if program.modules.get_by_name(module_name).is_some() {
            return Ok(());
        }

        let mut state = CompilationState::new(Input::ModuleName(module_name.to_owned()));

        for (_phase, phase_fn) in Phase::PHASES {
            let result = { phase_fn(self, &mut state, program) };
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

    pub fn run_until(
        &self,
        phase: Phase,
        input: Input,
        program: &mut Program,
    ) -> Result<CompilationState, Vec<Exception>> {
        let mut state = CompilationState::new(input);

        for (p, phase_fn) in Phase::PHASES {
            let result = { phase_fn(self, &mut state, program) };
            match result {
                Ok(_) => {}
                Err(exception) => {
                    let mut errors = state.errors.clone();
                    errors.push(exception);
                    return Err(errors);
                }
            }

            if p == phase {
                return Ok(state);
            }
        }

        Ok(state)
    }
}
