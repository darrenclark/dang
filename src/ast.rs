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

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ImportKind {
    Module {
        module_name: String,
    },
    Field {
        module_name: String,
        field_name: String,
    },
    AllFields {
        module_name: String,
    },
}

#[derive(Clone, Debug)]
pub enum NodeKind {
    SourceFile(Vec<Node>),
    Module(String),
    Import(ImportKind),
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
    Subscript {
        // object[key]
        object: Box<Node>,
        key: Box<Node>,
    },
    FieldAccess {
        // object.key
        object: Box<Node>,
        key: Box<Node>,
    },
    FunctionLiteral {
        arg_names: Vec<Node>,
        body: Box<Node>,
    },
    FunctionCall {
        function: Box<Node>,
        args: Vec<Node>,
    },
    Identifier(String),
    ListLiteral(Vec<Node>),
    DictLiteral(Vec<(Node, Node)>),
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
    Pipe {
        lhs: Box<Node>,
        rhs: Box<Node>,
    },
}

impl Node {
    pub fn iter(&self) -> NodeIterator {
        NodeIterator {
            node: self,
            returned_node: false,
            next_child_index: 0,
            inner_iter: None,
        }
    }

    pub fn child_at_index(&self, index: usize) -> Option<&Node> {
        match &self.kind {
            NodeKind::SourceFile(nodes) => nodes.get(index),
            NodeKind::Module(_) => None,
            NodeKind::Import(_) => None,
            NodeKind::Body(nodes) => nodes.get(index),
            NodeKind::Let { identifier, expr } => match index {
                0 => Some(identifier.as_ref()),
                1 => Some(expr.as_ref()),
                _ => None,
            },
            NodeKind::Var { identifier, expr } => match index {
                0 => Some(identifier.as_ref()),
                1 => Some(expr.as_ref()),
                _ => None,
            },
            NodeKind::Assignment { identifier, expr } => match index {
                0 => Some(identifier.as_ref()),
                1 => Some(expr.as_ref()),
                _ => None,
            },
            NodeKind::If {
                condition,
                body,
                else_branch: Some(else_branch),
            } => match index {
                0 => Some(condition.as_ref()),
                1 => Some(body.as_ref()),
                2 => Some(else_branch.as_ref()),
                _ => None,
            },
            NodeKind::If {
                condition,
                body,
                else_branch: None,
            } => match index {
                0 => Some(condition.as_ref()),
                1 => Some(body.as_ref()),
                _ => None,
            },
            NodeKind::For {
                var_name,
                enumerable,
                body,
            } => match index {
                0 => Some(var_name.as_ref()),
                1 => Some(enumerable.as_ref()),
                2 => Some(body.as_ref()),
                _ => None,
            },
            NodeKind::Subscript { object, key } | NodeKind::FieldAccess { object, key } => {
                match index {
                    0 => Some(object.as_ref()),
                    1 => Some(key.as_ref()),
                    _ => None,
                }
            }
            NodeKind::FunctionLiteral { arg_names, body } => {
                if index < arg_names.len() {
                    Some(&arg_names[index])
                } else if index == arg_names.len() {
                    Some(body.as_ref())
                } else {
                    None
                }
            }
            NodeKind::FunctionCall { function, args } => {
                if index == 0 {
                    Some(function.as_ref())
                } else if index < args.len() + 1 {
                    args.get(index - 1)
                } else {
                    None
                }
            }
            NodeKind::Identifier(_) => None,
            NodeKind::ListLiteral(_) => None,
            NodeKind::DictLiteral(_) => None,
            NodeKind::NilLiteral => None,
            NodeKind::BoolLiteral(_) => None,
            NodeKind::StringLiteral(_) => None,
            NodeKind::IntegerLiteral(_) => None,
            NodeKind::BinaryOp { op: _, lhs, rhs } => match index {
                0 => Some(lhs.as_ref()),
                1 => Some(rhs.as_ref()),
                _ => None,
            },
            NodeKind::UnaryOp { op: _, rhs } => match index {
                0 => Some(rhs.as_ref()),
                _ => None,
            },
            NodeKind::Pipe { lhs, rhs } => match index {
                0 => Some(lhs.as_ref()),
                1 => Some(rhs.as_ref()),
                _ => None,
            },
        }
    }
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
        NodeKind::Module(module_name) => {
            writeln!(f, "Module: {}", module_name)?;
        }
        NodeKind::Import(ImportKind::Module { module_name }) => {
            writeln!(f, "Import: Module {}", module_name)?;
        }
        NodeKind::Import(ImportKind::Field {
            module_name,
            field_name,
        }) => {
            writeln!(f, "Import: Field {}.{}", module_name, field_name)?;
        }
        NodeKind::Import(ImportKind::AllFields { module_name }) => {
            writeln!(f, "Import: AllFields {}.*", module_name)?;
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
        NodeKind::Subscript { object, key } => {
            writeln!(f, "Subscript:")?;
            fmt_node(object, depth + 1, "object", f)?;
            fmt_node(key, depth + 1, "key", f)?;
        }
        NodeKind::FieldAccess { object, key } => {
            writeln!(f, "FieldAccess:")?;
            fmt_node(object, depth + 1, "object", f)?;
            fmt_node(key, depth + 1, "key", f)?;
        }
        NodeKind::FunctionLiteral { arg_names, body } => {
            writeln!(f, "FunctionLiteral:")?;
            for a in arg_names {
                fmt_node(a, depth + 1, "arg_name", f)?;
            }
            fmt_node(body, depth + 1, "body", f)?;
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
        NodeKind::DictLiteral(pairs) => {
            writeln!(f, "DictLiteral")?;
            for (k, v) in pairs {
                fmt_node(k, depth + 1, "key", f)?;
                fmt_node(v, depth + 1, "value", f)?;
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
        NodeKind::Pipe { lhs, rhs } => {
            writeln!(f, "Pipe(|>)")?;
            fmt_node(lhs, depth + 1, "lhs", f)?;
            fmt_node(rhs, depth + 1, "rhs", f)?;
        }
    }
    Ok(())
}

pub struct NodeIterator<'a> {
    node: &'a Node,
    returned_node: bool,
    next_child_index: usize,
    inner_iter: Option<Box<NodeIterator<'a>>>,
}

impl<'a> Iterator for NodeIterator<'a> {
    type Item = &'a Node;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.returned_node {
            self.returned_node = true;
            self.next_child();
            return Some(self.node);
        }

        let mut result = None;

        if let Some(inner_iter) = &mut self.inner_iter {
            result = inner_iter.next();
        }

        if result.is_none() {
            self.next_child();
            if let Some(inner_iter) = &mut self.inner_iter {
                result = inner_iter.next();
            }
        }

        result
    }
}

impl<'a> NodeIterator<'a> {
    fn next_child(&mut self) {
        self.inner_iter = self
            .node
            .child_at_index(self.next_child_index)
            .map(|n| Box::new(n.iter()));
        self.next_child_index += 1;
    }
}
