#[derive(Clone, Debug)]
pub struct Node {
    pub kind: NodeKind,
}

#[derive(Clone, Debug)]
pub enum NodeKind {
    SourceFile(Vec<Node>),
    FunctionCall {
        function: Box<Node>,
        args: Vec<Node>,
    },
    Identifier(String),
    StringLiteral(String),
    IntegerLiteral(i64),
}
