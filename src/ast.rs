use std::rc::Rc;

#[derive(Clone, Debug)]
pub struct Node {
    pub kind: NodeKind,
    pub source: Source,
}

#[derive(Clone, Debug)]
pub struct Source {
    pub file: Rc<String>,
    pub line: usize,
    pub col: usize,
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
