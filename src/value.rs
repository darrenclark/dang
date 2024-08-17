use core::fmt;
use std::hash::Hash;
use std::mem::discriminant;
use std::rc::Rc;
use std::{cell::RefCell, collections::BTreeMap};

use escape8259::escape;
use lazy_static::lazy_static;
use pretty::termcolor::{Color, ColorSpec};
use pretty::RcDoc;
use ustr::Ustr;

use crate::ast::{BinOp, UnaryOp};
use crate::exception::{exception, Exception};
use crate::module::ModuleName;
use crate::vm::closure::Closure;
use crate::vm::function::Function;

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum Value {
    Nil,
    Symbol(Ustr),
    Bool(bool),
    String(Rc<String>),
    Integer(i64),
    NativeFunc(fn(&[Value]) -> Result<Value, Exception>),
    NativeClosure(NativeClosure),
    Function(Function),
    Closure(Closure),
    Tuple(Vec<Value>),
    List(Rc<Vec<Value>>),
    Dict(Rc<BTreeMap<Value, Value>>),
    Struct(ModuleName, Rc<BTreeMap<Ustr, Value>>),
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum ValuePath<'a> {
    At(Value),
    AtField(&'a str),
}

impl From<()> for Value {
    fn from(_: ()) -> Self {
        Self::Nil
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Self::string(value)
    }
}

impl<'a> From<&'a str> for Value {
    fn from(value: &'a str) -> Self {
        Self::string(value)
    }
}

impl From<i32> for Value {
    fn from(value: i32) -> Self {
        Self::Integer(value as i64)
    }
}

impl From<i64> for Value {
    fn from(value: i64) -> Self {
        Self::Integer(value)
    }
}

impl From<usize> for Value {
    fn from(value: usize) -> Self {
        Self::Integer(value as i64)
    }
}

impl<T: Into<Value>> From<Option<T>> for Value {
    fn from(value: Option<T>) -> Self {
        match value {
            Some(v) => v.into(),
            None => Self::Nil,
        }
    }
}

impl<T: Into<Value> + Clone> From<Vec<T>> for Value {
    fn from(value: Vec<T>) -> Self {
        let values = value.iter().map(|v| v.clone().into()).collect();
        Self::List(Rc::new(values))
    }
}

impl Value {
    pub fn unwrap_string(&self) -> &str {
        match self {
            Self::String(s) => s,
            _ => panic!("expected string, got {:?}", self),
        }
    }

    pub fn string<S: Into<String>>(s: S) -> Self {
        Value::String(Rc::new(s.into()))
    }

    pub fn dict(map: BTreeMap<Value, Value>) -> Self {
        Value::Dict(Rc::new(map))
    }

    pub fn struct_(module: ModuleName, map: BTreeMap<Ustr, Value>) -> Self {
        Value::Struct(module, Rc::new(map))
    }

    pub fn closure<F>(func: F) -> Self
    where
        F: FnMut(&[Value]) -> Result<Value, Exception> + 'static,
    {
        let c = NativeClosure {
            closure: Rc::new(RefCell::new(func)),
        };
        Value::NativeClosure(c)
    }

    pub fn truthy(&self) -> bool {
        match self {
            Self::Nil => false,
            Self::Bool(v) => *v,
            _ => true,
        }
    }

    pub fn is_enumerable(&self) -> bool {
        matches!(self, Self::String(_) | Self::List(_))
    }

    pub fn ensure_enumerable(&self, info: &'static str) -> Result<Value, Exception> {
        if self.is_enumerable() {
            Ok(self.clone())
        } else {
            exception!("{}: {:?} is not enumerable", info, self)
        }
    }

    pub fn enum_len(&self) -> usize {
        match self {
            Self::String(v) => v.chars().count(),
            Self::List(v) => v.len(),
            _ => panic!("{:?} is not enumerable", self),
        }
    }

    pub fn enum_at(&self, index: usize) -> Result<Value, Exception> {
        match self {
            Self::String(v) => {
                let result = v.chars().nth(index).map(|c| Value::string(String::from(c)));
                if let Some(result) = result {
                    Ok(result)
                } else {
                    let length = v.chars().count();
                    exception!(
                        "index {} out of bounds for string of length {}",
                        index,
                        length
                    )
                }
            }
            Self::List(v) => {
                let result = v.get(index);
                if let Some(result) = result {
                    Ok(result.clone())
                } else {
                    exception!(
                        "index {} out of bounds for string of length {}",
                        index,
                        v.len()
                    )
                }
            }
            _ => panic!("{:?} is not enumerable", self),
        }
    }

    pub fn at(&self, index: &Value) -> Result<Value, Exception> {
        if self.is_enumerable() {
            self.enum_at(index.to_index()?)
        } else if let Self::Tuple(v) = self {
            let i = index.to_index()?;
            if i >= v.len() {
                exception!("index {} out of bounds for tuple of size {}", i, v.len())
            }
            Ok(v.get(i).cloned().into())
        } else if let Self::Dict(v) = self {
            Ok(v.get(index).cloned().into())
        } else {
            exception!("cannot index in to {:?}", self)
        }
    }

    pub fn at_mut(&mut self, index: &Value) -> Result<&mut Value, Exception> {
        if let Self::List(ref mut v) = self {
            let i = index.to_index()?;
            if i >= v.len() {
                exception!("index {} out of bounds for list of length {}", i, v.len())
            }
            Ok(Rc::make_mut(v).get_mut(i).unwrap())
        } else if let Self::Tuple(v) = self {
            let i = index.to_index()?;
            if i >= v.len() {
                exception!("index {} out of bounds for tuple of size {}", i, v.len())
            }
            Ok(&mut v[i])
        } else if let Self::Dict(v) = self {
            Ok(Rc::make_mut(v).entry(index.clone()).or_insert(Value::Nil))
        } else {
            exception!("cannot index in to {:?}", self)
        }
    }

    pub fn at_field(&self, key: &str) -> Result<Value, Exception> {
        if let Self::Dict(v) = self {
            match v.get(&Value::from(key)) {
                None => exception!("field {:?} not found in {:?}", key, self),
                Some(v) => Ok(v.clone()),
            }
        } else if let Self::Struct(m, v) = self {
            match v.get(&Ustr::from(key)) {
                None => exception!("field `{}` not found in struct `{}`", key, m),
                Some(v) => Ok(v.clone()),
            }
        } else {
            exception!("cannot access field {:?} on {:?}", key, self)
        }
    }

    pub fn at_field_mut(&mut self, key: &str) -> Result<&mut Value, Exception> {
        if let Self::Dict(v) = self {
            match Rc::make_mut(v).get_mut(&Value::from(key)) {
                None => exception!("field {:?} not found in dict", key),
                Some(v) => Ok(v),
            }
        } else if let Self::Struct(m, v) = self {
            match Rc::make_mut(v).get_mut(&Ustr::from(key)) {
                None => exception!("field `{}` not found in struct `{}`", key, m),
                Some(v) => Ok(v),
            }
        } else {
            exception!("cannot access field {:?} on {:?}", key, self)
        }
    }

    pub fn at_path_mut<'a>(&mut self, path: &'a [ValuePath<'a>]) -> Result<&mut Value, Exception> {
        let mut current = self;
        for p in path {
            match p {
                ValuePath::At(v) => current = current.at_mut(v)?,
                ValuePath::AtField(f) => current = current.at_field_mut(f)?,
            }
        }
        Ok(current)
    }

    pub fn to_index(&self) -> Result<usize, Exception> {
        match self {
            Self::Integer(i) if *i >= 0 => Ok(*i as usize),
            _ => exception!("invalid index: {:?}", self),
        }
    }

    pub fn cast_to_int(&self) -> Result<Value, Exception> {
        match self {
            Self::Integer(_) => Ok(self.clone()),
            Self::Bool(true) => Ok(Value::Integer(1)),
            Self::Bool(false) => Ok(Value::Integer(0)),
            Self::String(contents) => match contents.parse::<i64>() {
                Ok(i) => Ok(Value::Integer(i)),
                Err(err) => exception!("failed to parse string to int: {}", err),
            },
            _ => exception!("cannot cast {:?} to int", self), // TODO: should nil convert to 0?
        }
    }

    pub fn cast_to_string(&self) -> Result<Value, Exception> {
        match self {
            Self::String(_) => Ok(self.clone()),
            Self::Integer(i) => Ok(Value::string(i.to_string())),
            Self::Bool(b) => Ok(Value::string(b.to_string())),
            _ => exception!("cannot cast {:?} to string", self),
        }
    }

    pub fn cast_to_symbol(&self) -> Result<Value, Exception> {
        match self {
            Self::String(contents) => Ok(Self::Symbol(Ustr::from(contents))),
            _ => exception!("cannot cast {:?} to symbol", self),
        }
    }

    pub fn iter<'a>(&'a self) -> Option<Box<dyn Iterator<Item = Value> + 'a>> {
        match self {
            Self::List(values) => Some(Box::new(values.iter().cloned())),
            Self::Dict(pairs) => Some(Box::new(
                pairs
                    .iter()
                    .map(|(k, v)| Self::Tuple(vec![k.clone(), v.clone()])),
            )),
            Self::String(contents) => Some(Box::new(
                contents.chars().map(|c| Value::string(String::from(c))),
            )),
            _ => None,
        }
    }

    pub fn into_iter(self) -> Option<Box<dyn Iterator<Item = Value>>> {
        match self {
            Self::List(values) => Some(Box::new((*values).clone().into_iter())),
            Self::Dict(pairs) => Some(Box::new(
                Rc::unwrap_or_clone(pairs)
                    .into_iter()
                    .map(|(k, v)| Self::Tuple(vec![k.clone(), v.clone()])),
            )),
            Self::String(contents) => Some(Box::new(
                contents
                    .chars()
                    .collect::<Vec<_>>()
                    .into_iter()
                    .map(|c| Value::string(String::from(c))),
            )),
            _ => None,
        }
    }

    pub fn bin_op(op: BinOp, lhs: &Value, rhs: &Value) -> Result<Value, Exception> {
        match (op, &lhs, &rhs) {
            // Addition
            (BinOp::Add, Value::String(l), Value::String(r)) => {
                Ok(Value::string(l.as_str().to_owned() + r.as_str()))
            }
            (BinOp::Add, Value::Integer(l), Value::Integer(r)) => Ok(Value::Integer(l + r)),
            (BinOp::Add, Value::List(l), Value::List(r)) => {
                let mut res = (**l).clone();
                res.extend(r.iter().cloned());
                Ok(Value::List(Rc::new(res)))
            }
            // Subtraction
            (BinOp::Sub, Value::Integer(l), Value::Integer(r)) => Ok(Value::Integer(l - r)),
            // Multiplication
            (BinOp::Mul, Value::Integer(l), Value::Integer(r)) => Ok(Value::Integer(l * r)),
            // Division
            (BinOp::Div, Value::Integer(l), Value::Integer(r)) => Ok(Value::Integer(l / r)),
            // || &&
            (BinOp::LogicalOr | BinOp::LogicalAnd, _, _) => unreachable!("handled in caller"),
            // == / !=
            (BinOp::Eq, l, r) => Ok(Value::Bool(l == r)),
            (BinOp::Neq, l, r) => Ok(Value::Bool(l != r)),
            // >=, <, etc.
            (BinOp::Gt, l, r) if discriminant(*l) == discriminant(*r) => Ok(Value::Bool(l > r)),
            (BinOp::Gte, l, r) if discriminant(*l) == discriminant(*r) => Ok(Value::Bool(l >= r)),
            (BinOp::Lt, l, r) if discriminant(*l) == discriminant(*r) => Ok(Value::Bool(l < r)),
            (BinOp::Lte, l, r) if discriminant(*l) == discriminant(*r) => Ok(Value::Bool(l <= r)),
            // Error
            (_, l, r) => {
                exception!("cannot apply `{:?}` to {:?} and {:?}", op, l, r)
            }
        }
    }

    pub fn unary_op(op: UnaryOp, rhs: &Value) -> Result<Value, Exception> {
        match (op, &rhs) {
            (UnaryOp::Neg, Value::Integer(r)) => Ok(Value::Integer(-r)),
            (UnaryOp::LogicalNeg, v) => Ok(Value::Bool(!v.truthy())),
            (_, v) => exception!("cannot {:?} {:?}", op, v),
        }
    }

    pub fn to_doc(&self) -> RcDoc<ColorSpec> {
        match self {
            Value::Nil => RcDoc::text("nil").annotate((*PRETTY_KEYWORD).clone()),
            Value::Symbol(s) => RcDoc::text(format!(":{}", s)).annotate((*PRETTY_SYMBOL).clone()),
            Value::Bool(true) => RcDoc::text("true").annotate((*PRETTY_KEYWORD).clone()),
            Value::Bool(false) => RcDoc::text("false").annotate((*PRETTY_KEYWORD).clone()),
            Value::String(s) => RcDoc::text(format!("\"{}\"", escape(s.as_ref())))
                .annotate((*PRETTY_STRING).clone()),
            Value::Integer(i) => RcDoc::as_string(i).annotate((*PRETTY_NUMBER).clone()),
            Value::NativeFunc(ptr) => RcDoc::text(format!("<NativeFunc({:?})>", ptr)),
            Value::NativeClosure(c) => RcDoc::text(format!("<NativeFunc({:?})>", c)),
            Value::Function(function) => RcDoc::text(format!("<Func({:?})>", function)),
            Value::Closure(function) => RcDoc::text(format!("<Func({:?})>", function)),
            Value::Tuple(list) if list.len() == 1 => RcDoc::text("(")
                .append(list[0].to_doc())
                .append(",")
                .append(RcDoc::text(")")),
            Value::Tuple(list) => RcDoc::text("(")
                .append(
                    RcDoc::intersperse(
                        list.iter().map(|x| x.to_doc()),
                        RcDoc::text(",").append(RcDoc::line()),
                    )
                    .nest(1)
                    .group(),
                )
                .append(RcDoc::text(")")),
            Value::List(list) => RcDoc::text("[")
                .append(
                    RcDoc::line_()
                        .append(
                            RcDoc::intersperse(
                                list.iter().map(|x| x.to_doc()),
                                RcDoc::text(",").append(RcDoc::line()),
                            )
                            .append(RcDoc::line_()),
                        )
                        .nest(2)
                        .group(),
                )
                .append(RcDoc::text("]")),
            Value::Dict(dict) => RcDoc::text("{")
                .append(
                    RcDoc::line_()
                        .append(
                            // TODO: format symbol keys differently once we move rest of Dang to
                            // use symbols for {x: 1} syntax
                            RcDoc::intersperse(
                                dict.iter()
                                    .map(|(k, v)| k.to_doc().append(" => ").append(v.to_doc())),
                                RcDoc::text(",").append(RcDoc::line()),
                            )
                            .append(RcDoc::line_()),
                        )
                        .nest(2)
                        .group(),
                )
                .append(RcDoc::text("}")),
            Value::Struct(module_name, dict) => RcDoc::text(module_name.short_name()).append(
                RcDoc::text("{")
                    .append(
                        RcDoc::line_()
                            .append(
                                // TODO: format symbol keys differently once we move rest of Dang to
                                // use symbols for {x: 1} syntax
                                RcDoc::intersperse(
                                    dict.iter().map(|(k, v)| {
                                        RcDoc::text(k.as_str())
                                            .append(":")
                                            .annotate((*PRETTY_SYMBOL).clone())
                                            .append(RcDoc::space())
                                            .append(v.to_doc())
                                    }),
                                    RcDoc::text(",").append(RcDoc::line()),
                                )
                                .append(RcDoc::line_()),
                            )
                            .nest(2)
                            .group(),
                    )
                    .append(RcDoc::text("}")),
            ),
        }
    }

    pub fn to_pretty(&self) -> String {
        let mut w = Vec::new();
        self.to_doc().render(80, &mut w).unwrap();
        String::from_utf8(w).unwrap()
    }

    pub fn unwrap_symbol(&self) -> Ustr {
        if let Self::Symbol(ustr) = self {
            *ustr
        } else {
            panic!("not a symbol")
        }
    }
}

lazy_static! {
    static ref PRETTY_NUMBER: ColorSpec = ColorSpec::new().set_fg(Some(Color::Yellow)).clone();
    static ref PRETTY_STRING: ColorSpec = ColorSpec::new().set_fg(Some(Color::Green)).clone();
    static ref PRETTY_SYMBOL: ColorSpec = ColorSpec::new().set_fg(Some(Color::Cyan)).clone();
    static ref PRETTY_KEYWORD: ColorSpec = ColorSpec::new().set_fg(Some(Color::Magenta)).clone();
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_pretty())
    }
}

#[derive(Clone)]
pub struct NativeClosure {
    pub closure: Rc<RefCell<dyn FnMut(&[Value]) -> Result<Value, Exception>>>,
}

impl fmt::Debug for NativeClosure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NativeClosure<{:p}>", self.closure)
    }
}

impl Eq for NativeClosure {}

impl PartialEq for NativeClosure {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.closure, &other.closure)
    }
}

impl Hash for NativeClosure {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        Rc::as_ptr(&self.closure).hash(state)
    }
}

impl Ord for NativeClosure {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let self_ptr = Rc::as_ptr(&self.closure);
        let other_ptr = Rc::as_ptr(&other.closure);
        self_ptr.cast::<()>().cmp(&other_ptr.cast::<()>())
    }
}

impl PartialOrd for NativeClosure {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
