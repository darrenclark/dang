use crate::{
    ast::{AstWalkerMut, Node, NodeId},
    module::ModuleId,
};

pub struct NodeIdsPass {
    next_node_id: NodeId,
}

impl NodeIdsPass {
    pub fn new(module_id: ModuleId) -> NodeIdsPass {
        NodeIdsPass {
            next_node_id: NodeId::first_in_module(module_id),
        }
    }

    pub fn run(&mut self, ast: &mut Node) {
        ast.walk_mut(self)
    }
}

impl AstWalkerMut for NodeIdsPass {
    fn enter_node(&mut self, node: &mut crate::ast::Node) {
        node.id = self.next_node_id;
        self.next_node_id = self.next_node_id.next_id();
    }
}
