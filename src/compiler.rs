pub mod compile_dependencies_phase;
mod desugar_pipe_phase;
mod emit_bytecode_phase;
pub mod finish_phase;
mod function_names_phase;
mod insert_prelude_phase;
pub mod node_ids_phase;
pub mod parse_ast_phase;
pub mod read_file_phase;
mod resolve_imports_phase;
mod resolve_structs_phase;
mod resolve_variables_phase;
pub mod struct_info_phase;
mod tag_return_exprs_phase;
pub mod tags;
mod validate_struct_fields_phase;

use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

use compile_dependencies_phase::compile_dependencies_phase;
use desugar_pipe_phase::desugar_pipe_phase;
use emit_bytecode_phase::emit_bytecode_phase;
use finish_phase::finish_phase;
use function_names_phase::function_names_phase;
use insert_prelude_phase::insert_prelude_phase;
use lazy_static::lazy_static;
use node_ids_phase::node_ids_phase;
use parse_ast_phase::parse_ast_phase;
use read_file_phase::read_file_phase;
use resolve_imports_phase::resolve_imports_phase;
use resolve_structs_phase::resolve_structs_phase;
use resolve_variables_phase::resolve_variables_phase;
use struct_info_phase::struct_info_phase;
use tag_return_exprs_phase::tag_return_exprs_phase;
use tags::Tags;
use validate_struct_fields_phase::validate_struct_fields_phase;

use crate::{
    ast::{ImportKind, Node, NodeId},
    exception::Exception,
    module::ModuleName,
    program::Program,
    scope::{UpvalueSource, VariableAllocation, VariableLocation},
    struct_info::StructInfo,
    value::Value,
    vm::{chunk::Chunk, function::Function},
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
    pub module_name: ModuleName,
    pub errors: Vec<Exception>,
    pub source_code: String,
    pub ast: Option<Node>,
    pub tags: Tags,
    pub imports: Vec<ResolvedImport>,
    pub exports: HashMap<String, VariableLocation>,
    pub constants: HashMap<String, Value>,
    pub variable_locations: HashMap<NodeId, VariableLocation>,
    pub variable_allocations: HashMap<NodeId, VariableAllocation>,
    pub closed_over_variables: HashSet<NodeId>,
    pub function_upvalues: HashMap<NodeId, Vec<UpvalueSource>>,
    pub struct_info: Option<StructInfo>,
    pub function: Function,
}

impl CompilationState {
    fn new(input: Input) -> CompilationState {
        let module_name = match &input {
            Input::ModuleName(n) => n.to_owned(),
            Input::SourceCode { text: _, name } => name.to_owned(),
        };

        CompilationState {
            input,
            module_name: module_name.into(),
            errors: Vec::new(),
            source_code: String::new(),
            ast: None,
            tags: Tags::new(),
            imports: Vec::new(),
            exports: HashMap::new(),
            constants: HashMap::new(),
            variable_locations: HashMap::new(),
            variable_allocations: HashMap::new(),
            closed_over_variables: HashSet::new(),
            function_upvalues: HashMap::new(),
            struct_info: None,
            function: Function::new(Chunk::new(), 0, vec![], "".to_owned()),
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
        self.module_name.is_std()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ResolvedImport {
    pub module_name: ModuleName,
    pub kind: ResolvedImportKind,
}

impl ResolvedImport {
    pub fn new(import_ast: &ImportKind) -> ResolvedImport {
        ResolvedImport {
            module_name: import_ast.module_name().into(),
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
        self.module_name.0.split('/').last()
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Phase {
    ReadFile,
    ParseAst,
    InsertPrelude,
    CompileDependencies,
    DesugarPipePhase,
    NodeIds,
    StructInfo,
    ResolveImports,
    ResolveVariables,
    ResolveStructs,
    ValidateStructFields,
    FunctionNames,
    TagReturnExprsPhase,
    EmitBytecode,
    Finish,
}

lazy_static! {
    static ref ALL_PHASES: Vec<(Phase, PhaseFn)> = vec![
        (Phase::ReadFile, read_file_phase),
        (Phase::ParseAst, parse_ast_phase),
        (Phase::InsertPrelude, insert_prelude_phase),
        (Phase::CompileDependencies, compile_dependencies_phase),
        (Phase::DesugarPipePhase, desugar_pipe_phase),
        (Phase::NodeIds, node_ids_phase),
        (Phase::StructInfo, struct_info_phase),
        (Phase::ResolveImports, resolve_imports_phase),
        (Phase::ResolveVariables, resolve_variables_phase),
        (Phase::ResolveStructs, resolve_structs_phase),
        (Phase::ValidateStructFields, validate_struct_fields_phase),
        (Phase::FunctionNames, function_names_phase),
        (Phase::TagReturnExprsPhase, tag_return_exprs_phase),
        (Phase::EmitBytecode, emit_bytecode_phase),
        (Phase::Finish, finish_phase),
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
        ImportKind::Field {
            module_name: "Std/Range".to_owned(),
            field_name: "range".to_owned()
        },
        ImportKind::Module {
            module_name: "Std/Set".to_owned()
        },
        ImportKind::Module {
            module_name: "Std/Stream".to_owned()
        },
    ];
}

#[derive(Debug)]
pub struct Compiler {
    pub implicit_imports: Vec<ImportKind>,
    pub module_search_paths: Vec<PathBuf>,
    phases: Vec<(Phase, PhaseFn)>,
}

impl Default for Compiler {
    fn default() -> Self {
        Compiler {
            implicit_imports: DEFAULT_IMPLICIT_IMPORTS.clone(),
            module_search_paths: Vec::default(),
            phases: (*ALL_PHASES).clone(),
        }
    }
}

impl Compiler {
    pub fn compile(&self, input: Input, program: &mut Program) -> Result<(), Vec<Exception>> {
        if program.get_module(&input.module_name().into()).is_some() {
            return Ok(());
        }

        let mut state = CompilationState::new(input);

        for (_phase, phase_fn) in &self.phases {
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

        for (p, phase_fn) in &self.phases {
            let result = { phase_fn(self, &mut state, program) };
            match result {
                Ok(_) => {}
                Err(exception) => {
                    let mut errors = state.errors.clone();
                    errors.push(exception);
                    return Err(errors);
                }
            }

            if *p == phase {
                return Ok(state);
            }
        }

        Ok(state)
    }

    pub fn remove_phase(&mut self, phase: Phase) {
        self.phases.retain(|(p, _)| *p != phase);
    }
}
