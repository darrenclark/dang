#[derive(Clone, Debug)]
pub struct Node {
    pub kind: NodeKind,
}

#[derive(Clone, Debug)]
pub enum NodeKind {
    SourceFile(Vec<Node>),
    Let {
        identifier: Box<Node>,
        expr: Box<Node>,
    },
    Var {
        identifier: Box<Node>,
        expr: Box<Node>,
    },
    Assignment {
        identifier: Box<Node>,
        expr: Box<Node>,
    },
    FunctionCall {
        function: Box<Node>,
        args: Vec<Node>,
    },
    Identifier(String),
    StringLiteral(String),
    IntegerLiteral(i64),
}
