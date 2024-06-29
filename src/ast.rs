use core::fmt;
use std::rc::Rc;

#[derive(Clone)]
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
    Add {
        lhs: Box<Node>,
        rhs: Box<Node>,
    },
    Sub {
        lhs: Box<Node>,
        rhs: Box<Node>,
    },
    Mul {
        lhs: Box<Node>,
        rhs: Box<Node>,
    },
    Div {
        lhs: Box<Node>,
        rhs: Box<Node>,
    },
}

impl fmt::Debug for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt_node(self, 0, "", f)
    }
}

fn fmt_node(node: &Node, depth: usize, label: &str, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", " ".repeat(depth * 2))?;
    if !label.is_empty() {
        write!(f, "{}: ", label)?;
    }
    match &node.kind {
        NodeKind::SourceFile(nodes) => {
            writeln!(f, "SourceFile:")?;
            for n in nodes {
                fmt_node(n, depth + 1, "", f)?;
            }
        }
        NodeKind::Let { identifier, expr } => {
            writeln!(f, "Let:")?;
            fmt_node(identifier, depth + 1, "name", f)?;
            fmt_node(expr, depth + 1, "value", f)?;
        }
        NodeKind::Var { identifier, expr } => {
            writeln!(f, "Var:")?;
            fmt_node(identifier, depth + 1, "name", f)?;
            fmt_node(expr, depth + 1, "value", f)?;
        }
        NodeKind::Assignment { identifier, expr } => {
            writeln!(f, "Assignment:")?;
            fmt_node(identifier, depth + 1, "name", f)?;
            fmt_node(expr, depth + 1, "value", f)?;
        }
        NodeKind::FunctionCall { function, args } => {
            writeln!(f, "FunctionCall:")?;
            fmt_node(function, depth + 1, "function", f)?;
            for a in args {
                fmt_node(a, depth + 1, "arg", f)?;
            }
        }
        NodeKind::Identifier(name) => {
            writeln!(f, "Identifier({})", name)?;
        }
        NodeKind::StringLiteral(string) => {
            writeln!(f, "StringLiteral(\"{}\")", string)?;
        }
        NodeKind::IntegerLiteral(value) => {
            writeln!(f, "IntegerLiteral({})", value)?;
        }
        NodeKind::Add { lhs, rhs } => {
            writeln!(f, "Add:")?;
            fmt_node(lhs, depth + 1, "", f)?;
            fmt_node(rhs, depth + 1, "", f)?;
        }
        NodeKind::Sub { lhs, rhs } => {
            writeln!(f, "Sub:")?;
            fmt_node(lhs, depth + 1, "", f)?;
            fmt_node(rhs, depth + 1, "", f)?;
        }
        NodeKind::Mul { lhs, rhs } => {
            writeln!(f, "Mul:")?;
            fmt_node(lhs, depth + 1, "", f)?;
            fmt_node(rhs, depth + 1, "", f)?;
        }
        NodeKind::Div { lhs, rhs } => {
            writeln!(f, "Div:")?;
            fmt_node(lhs, depth + 1, "", f)?;
            fmt_node(rhs, depth + 1, "", f)?;
        }
    }
    Ok(())
}
