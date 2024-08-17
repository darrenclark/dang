use crate::{
    ast::{AstWalkerMut, Node, NodeKind},
    exception::Exception,
    program::Program,
};

use super::{CompilationState, Compiler};

/// converts all NodeKind::Pipe into NodeKind::FunctionCall
pub fn desugar_pipe_phase(
    _compiler: &Compiler,
    compilation_state: &mut CompilationState,
    _program: &mut Program,
) -> Result<(), Exception> {
    let mut p = DesugarPipePhase { errors: Vec::new() };
    compilation_state.ast_mut().walk_mut(&mut p);

    compilation_state.errors.extend(p.errors);
    if !compilation_state.errors.is_empty() {
        return Err(Exception::new("Failed to desugar pipe operator".to_owned()));
    }

    Ok(())
}

struct DesugarPipePhase {
    errors: Vec<Exception>,
}

impl AstWalkerMut for DesugarPipePhase {
    fn enter_node(&mut self, node: &mut Node) {
        if let NodeKind::Pipe { lhs, rhs } = &node.kind {
            let (function, mut args) = match &rhs.kind {
                NodeKind::FunctionCall { function, args } => (function, args.clone()),
                _ => {
                    self.errors.push(Exception::at_node(
                        node,
                        "Pipe operator must be followed by a function call",
                    ));
                    return;
                }
            };

            args.insert(0, *lhs.clone());

            node.kind = NodeKind::FunctionCall {
                function: function.clone(),
                args,
            };
        }
    }
}
