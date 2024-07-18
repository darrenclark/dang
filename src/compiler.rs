pub mod compile_dependencies_phase;
pub mod finish_phase;
mod insert_prelude_phase;
pub mod node_ids_phase;
pub mod parse_ast_phase;
pub mod read_file_phase;
mod resolve_imports_phase;
mod resolve_variables_phase;

use std::collections::HashMap;

use compile_dependencies_phase::compile_dependencies_phase;
use finish_phase::finish_phase;
use insert_prelude_phase::insert_prelude_phase;
use lazy_static::lazy_static;
use node_ids_phase::node_ids_phase;
use parse_ast_phase::parse_ast_phase;
use read_file_phase::read_file_phase;
use resolve_imports_phase::resolve_imports_phase;
use resolve_variables_phase::resolve_variables_phase;

use crate::{
    ast::{ImportKind, Node, NodeId},
    interpreter::Exception,
    module::ModuleId,
    program::Program,
    scope::VariableLocation,
};

#[derive(Clone, Debug)]
pub enum Input {
    ModuleName(String),
    SourceCode { text: String, name: String },
}

impl Input {
    pub fn module_name(&self) -> &str {
        match self {
            Input::ModuleName(module_name) => module_name,
            Input::SourceCode { text: _, name } => name,
        }
    }
}

#[derive(Debug)]
pub struct CompilationState {
    pub input: Input,
    pub module_name: String,
    pub module_id: ModuleId,
    pub errors: Vec<Exception>,
    pub source_code: String,
    pub ast: Option<Node>,
    pub imports: Vec<ResolvedImport>,
    pub exports: HashMap<String, NodeId>,
    pub variable_locations: HashMap<NodeId, VariableLocation>,
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
            imports: Vec::new(),
            exports: HashMap::new(),
            variable_locations: HashMap::new(),
        }
    }

    pub fn ast(&self) -> &Node {
        self.ast.as_ref().unwrap()
    }

    pub fn ast_mut(&mut self) -> &mut Node {
        self.ast.as_mut().unwrap()
    }

    pub fn is_std(&self) -> bool {
        // TODO: Improve this
        self.module_name.starts_with("Std")
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ResolvedImport {
    pub module_id: ModuleId,
    pub module_name: String,
    pub kind: ResolvedImportKind,
}

impl ResolvedImport {
    pub fn new(module_id: ModuleId, import_ast: &ImportKind) -> ResolvedImport {
        ResolvedImport {
            module_id,
            module_name: import_ast.module_name().to_owned(),
            kind: match import_ast {
                ImportKind::Module { .. } => ResolvedImportKind::Module,
                ImportKind::Field {
                    module_name: _,
                    field_name,
                } => ResolvedImportKind::Field(field_name.to_owned()),
                ImportKind::AllFields { .. } => ResolvedImportKind::AllFields,
            },
        }
    }

    pub fn short_name(&self) -> Option<&str> {
        self.module_name.split('/').last()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ResolvedImportKind {
    /// import Std/Enum
    Module,
    /// import Std/Enum.map
    Field(String),
    /// import Std/Enum.*
    AllFields,
}

type PhaseFn = fn(&Compiler, &mut CompilationState, &mut Program) -> Result<(), Exception>;

#[derive(Debug, PartialEq, Eq)]
pub enum Phase {
    ReadFile,
    ParseAst,
    InsertPrelude,
    CompileDependencies,
    NodeIds,
    ResolveImports,
    ResolveVariables,
    Finish,
}

impl Phase {
    const PHASES: [(Self, PhaseFn); 8] = [
        (Self::ReadFile, read_file_phase),
        (Self::ParseAst, parse_ast_phase),
        (Self::InsertPrelude, insert_prelude_phase),
        (Self::CompileDependencies, compile_dependencies_phase),
        (Self::NodeIds, node_ids_phase),
        (Self::ResolveImports, resolve_imports_phase),
        (Self::ResolveVariables, resolve_variables_phase),
        (Self::Finish, finish_phase),
    ];
}

lazy_static! {
    static ref DEFAULT_IMPLICIT_IMPORTS: Vec<ImportKind> = vec![
        ImportKind::Module {
            module_name: "Std".to_owned()
        },
        ImportKind::AllFields {
            module_name: "Std/Assert".to_owned()
        },
        ImportKind::AllFields {
            module_name: "Std/Builtins".to_owned()
        },
        ImportKind::AllFields {
            module_name: "Std/Enum".to_owned()
        },
    ];
}

#[derive(Debug)]
pub struct Compiler {
    pub implicit_imports: Vec<ImportKind>,
}

impl Default for Compiler {
    fn default() -> Self {
        Compiler {
            implicit_imports: DEFAULT_IMPLICIT_IMPORTS.clone(),
        }
    }
}

impl Compiler {
    pub fn compile(&self, input: Input, program: &mut Program) -> Result<(), Vec<Exception>> {
        if program.modules.get_by_name(input.module_name()).is_some() {
            return Ok(());
        }

        let mut state = CompilationState::new(input);

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
