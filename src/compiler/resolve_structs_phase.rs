use crate::{
    ast::{AstWalker, Node, NodeId, NodeKind},
    interpreter::{exception, Exception},
    module::ModuleName,
    program::Program,
    scope::VariableAllocation,
    struct_info::StructInfo,
    value::Value,
};

use super::{CompilationState, Compiler};

#[derive(Debug)]
pub struct ReferencedStruct {
    pub struct_info: StructInfo,
}

pub fn resolve_structs_phase(
    _compiler: &Compiler,
    compilation_state: &mut CompilationState,
    program: &mut Program,
) -> Result<(), Exception> {
    let ast = compilation_state.ast().clone();

    let mut phase = ResolveStructsPhase {
        compilation_state,
        program,
        errored: false,
    };

    ast.walk(&mut phase);

    if phase.errored {
        exception!("failed to resolve structs")
    }

    Ok(())
}

struct ResolveStructsPhase<'a> {
    compilation_state: &'a mut CompilationState,
    program: &'a Program,
    errored: bool,
}

impl ResolveStructsPhase<'_> {
    fn lookup_variable_allocation(&self, module_node_id: NodeId) -> Option<VariableAllocation> {
        self.compilation_state
            .variable_allocations
            .get(&module_node_id)
            .cloned()
    }

    fn lookup_constant(&self, variable_allocation: &VariableAllocation) -> Option<Value> {
        match variable_allocation {
            VariableAllocation::Global { module, name } => {
                if module == &self.compilation_state.module_name {
                    self.compilation_state.constants.get(name.as_str()).cloned()
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn to_module_name(&self, value: &Value) -> Option<ModuleName> {
        if let Value::Symbol(s) = value {
            Some(ModuleName(*s))
        } else {
            None
        }
    }

    fn lookup_struct_info(&self, module_name: ModuleName) -> Option<StructInfo> {
        if module_name == self.compilation_state.module_name {
            self.compilation_state.struct_info.clone()
        } else {
            self.program
                .get_module(&module_name)
                .and_then(|m| m.struct_info.clone())
        }
    }
}

impl<'a> AstWalker for ResolveStructsPhase<'a> {
    fn enter_node(&mut self, node: &Node) {
        if let NodeKind::StructLiteral { module, .. } = &node.kind {
            let struct_info = self
                .lookup_variable_allocation(module.id)
                .and_then(|va| self.lookup_constant(&va))
                .and_then(|v| self.to_module_name(&v))
                .and_then(|mn| self.lookup_struct_info(mn));

            match struct_info {
                Some(struct_info) => {
                    self.compilation_state
                        .tags
                        .insert(node.id, ReferencedStruct { struct_info });
                }
                None => {
                    self.errored = true;
                    self.compilation_state.errors.push(Exception::at_node(
                        node,
                        "struct names must be known at compile time",
                    ));
                }
            };
        }
    }
}
