use crate::{
    ast::{AstWalker, Node, NodeId, NodeKind},
    exception::Exception,
    program::Program,
};

use super::{CompilationState, Compiler};

pub fn tag_return_exprs_phase(
    _compiler: &Compiler,
    compilation_state: &mut CompilationState,
    _program: &mut Program,
) -> Result<(), Exception> {
    let ast = compilation_state.ast().clone();

    let mut phase = TagReturnExprsPhase { compilation_state };

    ast.walk(&mut phase);

    Ok(())
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct IsReturnExpr {
    pub function: NodeId,
}

struct TagReturnExprsPhase<'a> {
    compilation_state: &'a mut CompilationState,
}

impl TagReturnExprsPhase<'_> {
    fn tag_return_exprs(&mut self, function: &Node, node: &Node) {
        let node_to_check = match &node.kind {
            NodeKind::SourceFile(nodes) if !nodes.is_empty() => nodes.last().unwrap(),
            NodeKind::Body(nodes) if !nodes.is_empty() => nodes.last().unwrap(),
            NodeKind::If { .. } => node,
            NodeKind::Match { .. } => node,
            _ => return,
        };

        match &node_to_check.kind {
            NodeKind::If {
                condition: _,
                body,
                else_branch,
            } => {
                self.tag_return_exprs(function, body);
                if let Some(else_branch) = else_branch {
                    self.tag_return_exprs(function, else_branch);
                }
            }
            NodeKind::Match { expr: _, cases } => {
                for case in cases {
                    if let NodeKind::MatchCase {
                        pattern: _,
                        guard: _,
                        body,
                    } = &case.kind
                    {
                        self.tag_return_exprs(function, body);
                    }
                }
            }
            _ => {
                self.compilation_state.tags.insert(
                    node_to_check.id,
                    IsReturnExpr {
                        function: function.id,
                    },
                );
            }
        }
    }
}

impl AstWalker for TagReturnExprsPhase<'_> {
    fn enter_node(&mut self, node: &Node) {
        match &node.kind {
            NodeKind::SourceFile(_) => {
                self.tag_return_exprs(node, node);
            }
            NodeKind::FunctionLiteral { arg_names: _, body } => {
                self.tag_return_exprs(node, body);
            }
            _ => {}
        }
    }
}
