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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    LogicalOr,
    LogicalAnd,
    Eq,
    Neq,
    Gt,
    Gte,
    Lt,
    Lte,
}

#[derive(Clone, Copy, Debug)]
pub enum UnaryOp {
    Neg,
    LogicalNeg,
}

#[derive(Clone, Debug)]
pub enum NodeKind {
    SourceFile(Vec<Node>),
    Body(Vec<Node>),
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
    If {
        condition: Box<Node>,
        body: Box<Node>,
        else_branch: Option<Box<Node>>,
    },
    For {
        var_name: Box<Node>,
        enumerable: Box<Node>,
        body: Box<Node>,
    },
    FunctionLiteral {
        arg_names: Vec<Node>,
        body: Vec<Node>,
    },
    FunctionCall {
        function: Box<Node>,
        args: Vec<Node>,
    },
    Identifier(String),
    ListLiteral(Vec<Node>),
    NilLiteral,
    BoolLiteral(bool),
    StringLiteral(String),
    IntegerLiteral(i64),
    BinaryOp {
        op: BinOp,
        lhs: Box<Node>,
        rhs: Box<Node>,
    },
    UnaryOp {
        op: UnaryOp,
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
        NodeKind::Body(nodes) => {
            writeln!(f, "Body:")?;
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
        NodeKind::If {
            condition,
            body,
            else_branch,
        } => {
            writeln!(f, "If:")?;
            fmt_node(condition, depth + 1, "cond", f)?;
            fmt_node(body, depth + 1, "body", f)?;
            if let Some(else_branch) = else_branch {
                fmt_node(else_branch, depth + 1, "else", f)?;
            }
        }
        NodeKind::For {
            var_name,
            enumerable,
            body,
        } => {
            writeln!(f, "For:")?;
            fmt_node(var_name, depth + 1, "var_name", f)?;
            fmt_node(enumerable, depth + 1, "enumerable", f)?;
            fmt_node(body, depth + 1, "body", f)?;
        }
        NodeKind::FunctionLiteral { arg_names, body } => {
            writeln!(f, "FunctionLiteral:")?;
            for a in arg_names {
                fmt_node(a, depth + 1, "arg_name", f)?;
            }
            writeln!(f, "{}body:", " ".repeat((depth + 1) * 2))?;
            for n in body {
                fmt_node(n, depth + 2, "", f)?;
            }
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
        NodeKind::ListLiteral(elements) => {
            writeln!(f, "ListLiteral")?;
            for e in elements {
                fmt_node(e, depth + 1, "element", f)?;
            }
        }
        NodeKind::NilLiteral => {
            writeln!(f, "NilLiteral(nil)")?;
        }
        NodeKind::BoolLiteral(value) => {
            writeln!(f, "BoolLiteral({})", value)?;
        }
        NodeKind::StringLiteral(string) => {
            writeln!(f, "StringLiteral(\"{}\")", string)?;
        }
        NodeKind::IntegerLiteral(value) => {
            writeln!(f, "IntegerLiteral({})", value)?;
        }
        NodeKind::BinaryOp { op, lhs, rhs } => {
            writeln!(f, "BinaryOp({:?}):", op)?;
            fmt_node(lhs, depth + 1, "", f)?;
            fmt_node(rhs, depth + 1, "", f)?;
        }
        NodeKind::UnaryOp { op, rhs } => {
            writeln!(f, "UnaryOp({:?}):", op)?;
            fmt_node(rhs, depth + 1, "", f)?;
        }
    }
    Ok(())
}
