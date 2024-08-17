use crate::{
    ast::{AstWalkerMut, NodeId},
    exception::Exception,
    program::Program,
};

use super::{CompilationState, Compiler};

pub fn node_ids_phase(
    _compiler: &Compiler,
    compilation_state: &mut CompilationState,
    _program: &mut Program,
) -> Result<(), Exception> {
    let mut p = NodeIdsPhase {
        next_node_id: NodeId::first_in_module(compilation_state.module_name),
    };
    compilation_state.ast_mut().walk_mut(&mut p);

    Ok(())
}

struct NodeIdsPhase {
    next_node_id: NodeId,
}

impl AstWalkerMut for NodeIdsPhase {
    fn enter_node(&mut self, node: &mut crate::ast::Node) {
        node.id = self.next_node_id;
        self.next_node_id = self.next_node_id.next_id();
    }
}
