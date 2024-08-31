use crate::{
    ast::{AstWalker, Node, NodeKind},
    exception::Exception,
    program::Program,
};

use super::{variable::VariableRef, CompilationState, Compiler};

#[derive(Debug)]
pub struct FunctionName {
    pub name: String,
}

pub fn function_names_phase(
    _compiler: &Compiler,
    compilation_state: &mut CompilationState,
    _program: &mut Program,
) -> Result<(), Exception> {
    let ast = compilation_state.ast().clone();

    let mut phase = FunctionNamesPhase { compilation_state };

    ast.walk(&mut phase);

    Ok(())
}

struct FunctionNamesPhase<'a> {
    compilation_state: &'a mut CompilationState,
}

impl FunctionNamesPhase<'_> {
    fn namespaced(&self, name: &str) -> String {
        format!("{}.{}", self.compilation_state.module_name.0, name)
    }

    fn local(&self, name: &str, line: usize) -> String {
        format!(
            "{} - {} (at line {})",
            self.compilation_state.module_name.0, name, line
        )
    }
}

impl<'a> AstWalker for FunctionNamesPhase<'a> {
    fn enter_node(&mut self, node: &Node) {
        match &node.kind {
            NodeKind::SourceFile { .. } => {
                self.compilation_state.tags.insert(
                    node.id,
                    FunctionName {
                        name: self.namespaced("(main)"),
                    },
                );
            }
            NodeKind::Builtin { identifier } => {
                self.compilation_state.tags.insert(
                    node.id,
                    FunctionName {
                        name: self.namespaced(identifier.unwrap_identifier()),
                    },
                );
            }
            NodeKind::Let {
                ref pattern,
                ref expr,
            }
            | NodeKind::Var {
                ref pattern,
                ref expr,
            } => match (&pattern.kind, &expr.kind) {
                (NodeKind::PatternIdentifier { identifier }, NodeKind::FunctionLiteral { .. }) => {
                    let name = if self
                        .compilation_state
                        .tags
                        .get::<VariableRef>(pattern.id)
                        .unwrap()
                        .allocation
                        .is_global()
                    {
                        self.namespaced(identifier.unwrap_identifier())
                    } else {
                        self.local(identifier.unwrap_identifier(), identifier.source.start.line)
                    };
                    self.compilation_state
                        .tags
                        .insert(expr.id, FunctionName { name });
                }
                _ => (),
            },
            NodeKind::FunctionLiteral { .. } => {
                if self
                    .compilation_state
                    .tags
                    .get::<FunctionName>(node.id)
                    .is_none()
                {
                    let name = format!("(anonymous at line {})", node.source.start.line);
                    self.compilation_state.tags.insert(
                        node.id,
                        FunctionName {
                            name: self.namespaced(&name),
                        },
                    );
                }
            }
            _ => {}
        }
    }
}
