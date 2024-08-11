use core::fmt;
use std::{collections::BTreeMap, rc::Rc};

use ustr::Ustr;

use crate::{module::ModuleName, value::Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeId {
    module: ModuleName,
    local_id: u32,
}

pub const UNSPECIFIED_NODE_LOCAL_ID: u32 = u32::MAX;

impl NodeId {
    pub fn first_in_module(module: ModuleName) -> NodeId {
        NodeId {
            module,
            local_id: 0,
        }
    }

    pub fn next_id(&self) -> NodeId {
        NodeId {
            module: self.module,
            local_id: self.local_id + 1,
        }
    }

    pub fn raw_id(&self) -> (Ustr, u32) {
        (self.module.0, self.local_id)
    }

    pub fn module(&self) -> ModuleName {
        self.module
    }
}

impl Default for NodeId {
    fn default() -> Self {
        NodeId {
            module: ModuleName::default(),
            local_id: UNSPECIFIED_NODE_LOCAL_ID,
        }
    }
}

#[derive(Clone)]
pub struct Node {
    pub id: NodeId,
    pub kind: NodeKind,
    pub source: Source,
}

#[derive(Clone, Debug)]
pub struct Source {
    pub file: Rc<String>,
    pub line: usize,
    pub col: usize,
}

impl fmt::Display for Source {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let components = self.file.split("/").collect::<Vec<_>>();
        if components.len() > 2 {
            write!(
                f,
                ".../{}/{}:{}",
                components[components.len() - 2],
                components[components.len() - 1],
                self.line
            )
        } else {
            write!(f, "{}:{}", self.file, self.line)
        }
    }
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

impl ImportKind {
    pub fn module_name(&self) -> &str {
        match self {
            ImportKind::Module { module_name } => module_name,
            ImportKind::Field {
                module_name,
                field_name: _,
            } => module_name,
            ImportKind::AllFields { module_name } => module_name,
        }
    }
}

#[derive(Clone, Debug)]
pub enum NodeKind {
    SourceFile(Vec<Node>),
    Module(String),
    Import(ImportKind),
    StructDef {
        // field_name => default_value (?)
        fields: Vec<(Node, Option<Node>)>,
    },
    Body(Vec<Node>),
    Builtin {
        identifier: Box<Node>,
    },
    Let {
        pattern: Box<Node>,
        expr: Box<Node>,
    },
    Var {
        pattern: Box<Node>,
        expr: Box<Node>,
    },
    Assignment {
        pattern: Box<Node>,
        expr: Box<Node>,
    },
    FieldAssignment {
        variable_ref: Box<Node>,
        path: Vec<Node>,
        expr: Box<Node>,
    },
    FieldAssignmentPathSubscript {
        // (...)[key] = ...
        key: Box<Node>,
    },
    FieldAssignmentPathField {
        // (...).key = ...
        key: Box<Node>,
    },
    If {
        condition: Box<Node>,
        body: Box<Node>,
        else_branch: Option<Box<Node>>,
    },
    For {
        pattern: Box<Node>,
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
    VariableRef {
        identifier: Box<Node>,
    },
    PatternIdentifier {
        identifier: Box<Node>,
    },
    PatternTuple {
        elements: Vec<Node>,
    },
    Identifier(String),
    TupleLiteral(Vec<Node>),
    ListLiteral(Vec<Node>),
    DictLiteral(Vec<(Node, Node)>),
    StructLiteral {
        module: Box<Node>,
        fields: Vec<(Node, Node)>,
    },
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
    /// Iterates over every node in tree
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
            NodeKind::StructDef { fields } => {
                let mut i = 0;
                for (name, default_value) in fields {
                    if index == i {
                        return Some(name);
                    }
                    i += 1;
                    if index == i && default_value.is_some() {
                        return default_value.as_ref();
                    }
                    if default_value.is_some() {
                        i += 1
                    }
                }
                None
            }
            NodeKind::Body(nodes) => nodes.get(index),
            NodeKind::Builtin { identifier } => match index {
                0 => Some(identifier.as_ref()),
                _ => None,
            },
            NodeKind::Let {
                pattern: identifier,
                expr,
            } => match index {
                0 => Some(identifier.as_ref()),
                1 => Some(expr.as_ref()),
                _ => None,
            },
            NodeKind::Var {
                pattern: identifier,
                expr,
            } => match index {
                0 => Some(identifier.as_ref()),
                1 => Some(expr.as_ref()),
                _ => None,
            },
            NodeKind::Assignment {
                pattern: identifier,
                expr,
            } => match index {
                0 => Some(identifier.as_ref()),
                1 => Some(expr.as_ref()),
                _ => None,
            },
            NodeKind::FieldAssignment {
                variable_ref,
                path,
                expr,
            } => {
                if index == 0 {
                    Some(variable_ref.as_ref())
                } else if index < path.len() + 1 {
                    path.get(index - 1)
                } else if index == path.len() + 1 {
                    Some(expr.as_ref())
                } else {
                    None
                }
            }
            NodeKind::FieldAssignmentPathSubscript { key } => {
                if index == 0 {
                    Some(key.as_ref())
                } else {
                    None
                }
            }
            NodeKind::FieldAssignmentPathField { key } => {
                if index == 0 {
                    Some(key.as_ref())
                } else {
                    None
                }
            }
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
                pattern: var_name,
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
            NodeKind::VariableRef { identifier } => {
                if index == 0 {
                    Some(identifier.as_ref())
                } else {
                    None
                }
            }
            NodeKind::PatternIdentifier { identifier } => {
                if index == 0 {
                    Some(identifier.as_ref())
                } else {
                    None
                }
            }
            NodeKind::PatternTuple { elements } => {
                if index < elements.len() {
                    Some(&elements[index])
                } else {
                    None
                }
            }
            NodeKind::Identifier(_) => None,
            NodeKind::TupleLiteral(items) => {
                if index < items.len() {
                    Some(&items[index])
                } else {
                    None
                }
            }
            NodeKind::ListLiteral(items) => {
                if index < items.len() {
                    Some(&items[index])
                } else {
                    None
                }
            }
            NodeKind::DictLiteral(entries) => {
                let vec_index = index / 2;
                let tuple_elem = index % 2;
                if vec_index < entries.len() {
                    if tuple_elem == 0 {
                        Some(&entries[vec_index].0)
                    } else {
                        Some(&entries[vec_index].1)
                    }
                } else {
                    None
                }
            }
            NodeKind::StructLiteral { module, fields } => {
                if index == 0 {
                    Some(module)
                } else {
                    let vec_index = (index - 1) / 2;
                    let tuple_elem = (index - 1) % 2;
                    if vec_index < fields.len() {
                        if tuple_elem == 0 {
                            Some(&fields[vec_index].0)
                        } else {
                            Some(&fields[vec_index].1)
                        }
                    } else {
                        None
                    }
                }
            }
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

    pub fn child_at_index_mut(&mut self, index: usize) -> Option<&mut Node> {
        match &mut self.kind {
            NodeKind::SourceFile(nodes) => nodes.get_mut(index),
            NodeKind::Module(_) => None,
            NodeKind::Import(_) => None,
            NodeKind::StructDef { fields } => {
                let mut i = 0;
                for (name, default_value) in fields.iter_mut() {
                    if index == i {
                        return Some(name);
                    }
                    i += 1;
                    if index == i && default_value.is_some() {
                        return default_value.as_mut();
                    }
                    if default_value.is_some() {
                        i += 1
                    }
                }
                None
            }
            NodeKind::Body(nodes) => nodes.get_mut(index),
            NodeKind::Builtin { identifier } => match index {
                0 => Some(identifier.as_mut()),
                _ => None,
            },
            NodeKind::Let {
                pattern: identifier,
                expr,
            } => match index {
                0 => Some(identifier.as_mut()),
                1 => Some(expr.as_mut()),
                _ => None,
            },
            NodeKind::Var {
                pattern: identifier,
                expr,
            } => match index {
                0 => Some(identifier.as_mut()),
                1 => Some(expr.as_mut()),
                _ => None,
            },
            NodeKind::Assignment {
                pattern: identifier,
                expr,
            } => match index {
                0 => Some(identifier.as_mut()),
                1 => Some(expr.as_mut()),
                _ => None,
            },
            NodeKind::FieldAssignment {
                variable_ref,
                path,
                expr,
            } => {
                if index == 0 {
                    Some(variable_ref.as_mut())
                } else if index < path.len() + 1 {
                    path.get_mut(index - 1)
                } else if index == path.len() + 1 {
                    Some(expr.as_mut())
                } else {
                    None
                }
            }
            NodeKind::FieldAssignmentPathSubscript { key } => {
                if index == 0 {
                    Some(key.as_mut())
                } else {
                    None
                }
            }
            NodeKind::FieldAssignmentPathField { key } => {
                if index == 0 {
                    Some(key.as_mut())
                } else {
                    None
                }
            }
            NodeKind::If {
                condition,
                body,
                else_branch: Some(else_branch),
            } => match index {
                0 => Some(condition.as_mut()),
                1 => Some(body.as_mut()),
                2 => Some(else_branch.as_mut()),
                _ => None,
            },
            NodeKind::If {
                condition,
                body,
                else_branch: None,
            } => match index {
                0 => Some(condition.as_mut()),
                1 => Some(body.as_mut()),
                _ => None,
            },
            NodeKind::For {
                pattern: var_name,
                enumerable,
                body,
            } => match index {
                0 => Some(var_name.as_mut()),
                1 => Some(enumerable.as_mut()),
                2 => Some(body.as_mut()),
                _ => None,
            },
            NodeKind::Subscript { object, key } | NodeKind::FieldAccess { object, key } => {
                match index {
                    0 => Some(object.as_mut()),
                    1 => Some(key.as_mut()),
                    _ => None,
                }
            }
            NodeKind::FunctionLiteral { arg_names, body } => {
                if index < arg_names.len() {
                    arg_names.get_mut(index)
                } else if index == arg_names.len() {
                    Some(body.as_mut())
                } else {
                    None
                }
            }
            NodeKind::FunctionCall { function, args } => {
                if index == 0 {
                    Some(function.as_mut())
                } else if index < args.len() + 1 {
                    args.get_mut(index - 1)
                } else {
                    None
                }
            }
            NodeKind::VariableRef { identifier } => {
                if index == 0 {
                    Some(identifier.as_mut())
                } else {
                    None
                }
            }
            NodeKind::PatternIdentifier { identifier } => {
                if index == 0 {
                    Some(identifier.as_mut())
                } else {
                    None
                }
            }
            NodeKind::PatternTuple { elements } => {
                if index < elements.len() {
                    elements.get_mut(index)
                } else {
                    None
                }
            }
            NodeKind::Identifier(_) => None,
            NodeKind::TupleLiteral(items) => {
                if index < items.len() {
                    items.get_mut(index)
                } else {
                    None
                }
            }
            NodeKind::ListLiteral(items) => {
                if index < items.len() {
                    items.get_mut(index)
                } else {
                    None
                }
            }
            NodeKind::DictLiteral(entries) => {
                let vec_index = index / 2;
                let tuple_elem = index % 2;
                if vec_index < entries.len() {
                    if tuple_elem == 0 {
                        entries.get_mut(vec_index).map(|e| &mut e.0)
                    } else {
                        entries.get_mut(vec_index).map(|e| &mut e.1)
                    }
                } else {
                    None
                }
            }
            NodeKind::StructLiteral { module, fields } => {
                if index == 0 {
                    Some(module)
                } else {
                    let vec_index = (index - 1) / 2;
                    let tuple_elem = (index - 1) % 2;
                    if vec_index < fields.len() {
                        if tuple_elem == 0 {
                            fields.get_mut(vec_index).map(|e| &mut e.0)
                        } else {
                            fields.get_mut(vec_index).map(|e| &mut e.1)
                        }
                    } else {
                        None
                    }
                }
            }
            NodeKind::NilLiteral => None,
            NodeKind::BoolLiteral(_) => None,
            NodeKind::StringLiteral(_) => None,
            NodeKind::IntegerLiteral(_) => None,
            NodeKind::BinaryOp { op: _, lhs, rhs } => match index {
                0 => Some(lhs.as_mut()),
                1 => Some(rhs.as_mut()),
                _ => None,
            },
            NodeKind::UnaryOp { op: _, rhs } => match index {
                0 => Some(rhs.as_mut()),
                _ => None,
            },
            NodeKind::Pipe { lhs, rhs } => match index {
                0 => Some(lhs.as_mut()),
                1 => Some(rhs.as_mut()),
                _ => None,
            },
        }
    }

    pub fn unwrap_identifier(&self) -> &str {
        if let NodeKind::Identifier(name) = &self.kind {
            name
        } else {
            panic!("not an identifier: {:?}", self)
        }
    }

    pub fn is_function_literal(&self) -> bool {
        matches!(self.kind, NodeKind::FunctionLiteral { .. })
    }

    pub fn find_by_id(&self, id: NodeId) -> Option<&Node> {
        // TOOD: binary-ish search instead?
        if self.id.module == id.module {
            self.iter().find(|n| n.id == id)
        } else {
            None
        }
    }

    /// If AST compromised of compile time constants, returns the Value
    pub fn compile_time_value(&self) -> Option<Value> {
        match &self.kind {
            NodeKind::ListLiteral(elements) => {
                let mut res = Vec::with_capacity(elements.len());
                for e in elements {
                    if let Some(v) = e.compile_time_value() {
                        res.push(v)
                    } else {
                        return None;
                    }
                }
                Some(Value::List(Rc::new(res)))
            }
            NodeKind::TupleLiteral(elements) => {
                let mut res = Vec::with_capacity(elements.len());
                for e in elements {
                    if let Some(v) = e.compile_time_value() {
                        res.push(v)
                    } else {
                        return None;
                    }
                }
                Some(Value::Tuple(res))
            }
            NodeKind::DictLiteral(key_values) => {
                let mut map = BTreeMap::new();
                for (k, v) in key_values {
                    match (k.compile_time_value(), v.compile_time_value()) {
                        (Some(k), Some(v)) => {
                            map.insert(k, v);
                        }
                        _ => return None,
                    }
                }
                Some(Value::dict(map))
            }
            NodeKind::NilLiteral => Some(Value::Nil),
            NodeKind::BoolLiteral(v) => Some(Value::Bool(*v)),
            NodeKind::StringLiteral(contents) => Some(Value::String(contents.to_owned())),
            NodeKind::IntegerLiteral(i) => Some(Value::Integer(*i)),
            _ => None,
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
        NodeKind::StructDef { fields } => {
            writeln!(f, "StructDef")?;
            for (k, v) in fields {
                fmt_node(k, depth + 1, "name", f)?;
                if let Some(v) = v {
                    fmt_node(v, depth + 1, "default", f)?;
                }
            }
        }
        NodeKind::Body(nodes) => {
            writeln!(f, "Body:")?;
            for n in nodes {
                fmt_node(n, depth + 1, "", f)?;
            }
        }
        NodeKind::Builtin { identifier } => {
            writeln!(f, "Builtin:")?;
            fmt_node(identifier, depth + 1, "identifier", f)?;
        }
        NodeKind::Let { pattern, expr } => {
            writeln!(f, "Let:")?;
            fmt_node(pattern, depth + 1, "pattern", f)?;
            fmt_node(expr, depth + 1, "value", f)?;
        }
        NodeKind::Var { pattern, expr } => {
            writeln!(f, "Var:")?;
            fmt_node(pattern, depth + 1, "pattern", f)?;
            fmt_node(expr, depth + 1, "value", f)?;
        }
        NodeKind::Assignment { pattern, expr } => {
            writeln!(f, "Assignment:")?;
            fmt_node(pattern, depth + 1, "pattern", f)?;
            fmt_node(expr, depth + 1, "value", f)?;
        }
        NodeKind::FieldAssignment {
            variable_ref,
            path,
            expr,
        } => {
            writeln!(f, "FieldAssignment:")?;
            fmt_node(variable_ref, depth + 1, "variable_ref", f)?;
            for p in path {
                fmt_node(p, depth + 1, "path", f)?;
            }
            fmt_node(expr, depth + 1, "value", f)?;
        }
        NodeKind::FieldAssignmentPathSubscript { key } => {
            writeln!(f, "FieldAssignmentPathSubscript:")?;
            fmt_node(key, depth + 1, "key", f)?;
        }
        NodeKind::FieldAssignmentPathField { key } => {
            writeln!(f, "FieldAssignmentPathField:")?;
            fmt_node(key, depth + 1, "key", f)?;
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
            pattern,
            enumerable,
            body,
        } => {
            writeln!(f, "For:")?;
            fmt_node(pattern, depth + 1, "pattern", f)?;
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
        NodeKind::VariableRef { identifier } => {
            writeln!(f, "VariableRef:")?;
            fmt_node(identifier, depth + 1, "identifier", f)?;
        }
        NodeKind::PatternIdentifier { identifier } => {
            writeln!(f, "PatternIdentifier:")?;
            fmt_node(identifier, depth + 1, "identifier", f)?;
        }
        NodeKind::PatternTuple { elements } => {
            writeln!(f, "PatternTuple")?;
            for e in elements {
                fmt_node(e, depth + 1, "element", f)?;
            }
        }
        NodeKind::Identifier(name) => {
            writeln!(f, "Identifier({})", name)?;
        }
        NodeKind::TupleLiteral(elements) => {
            writeln!(f, "TupleLiteral")?;
            for e in elements {
                fmt_node(e, depth + 1, "element", f)?;
            }
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
        NodeKind::StructLiteral { module, fields } => {
            writeln!(f, "StructLiteral")?;
            fmt_node(module, depth + 1, "module", f)?;
            for (k, v) in fields {
                fmt_node(k, depth + 1, "field", f)?;
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

pub trait AstWalker {
    fn enter_node(&mut self, node: &Node);
    fn walk_children(&mut self, _node: &Node) -> bool {
        true
    }
    fn exit_node(&mut self, _node: &Node) {}
}

pub trait AstWalkerMut {
    fn enter_node(&mut self, node: &mut Node);
    fn walk_children(&mut self, _node: &Node) -> bool {
        true
    }
    fn exit_node(&mut self, _node: &mut Node) {}
}

impl Node {
    pub fn walk<W: AstWalker>(&self, walker: &mut W) {
        walker.enter_node(self);

        if walker.walk_children(self) {
            let mut i = 0;
            while let Some(child) = self.child_at_index(i) {
                child.walk(walker);
                i += 1;
            }
        }

        walker.exit_node(self);
    }

    pub fn walk_mut<W: AstWalkerMut>(&mut self, walker: &mut W) {
        walker.enter_node(self);

        if walker.walk_children(self) {
            let mut i = 0;
            while let Some(child) = self.child_at_index_mut(i) {
                child.walk_mut(walker);
                i += 1;
            }
        }

        walker.exit_node(self);
    }
}
