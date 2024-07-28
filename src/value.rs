use core::fmt;
use std::hash::Hash;
use std::rc::Rc;
use std::{cell::RefCell, collections::BTreeMap};

use ustr::Ustr;

use crate::interpreter::Environment;
use crate::module::ModuleName;
use crate::{
    ast::Node,
    interpreter::{exception, Exception},
};

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum Value {
    Nil,
    Symbol(Ustr),
    Bool(bool),
    String(String),
    Integer(i64),
    NativeFunc(fn(&[Value]) -> Result<Value, Exception>),
    NativeClosure(NativeClosure),
    Func(FunctionLiteral),
    Tuple(Vec<Value>),
    List(Rc<Vec<Value>>),
    Dict(BTreeMap<Value, Value>),
    Struct(ModuleName, BTreeMap<Ustr, Value>),
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
        Self::String(value)
    }
}

impl<'a> From<&'a str> for Value {
    fn from(value: &'a str) -> Self {
        Self::String(value.to_owned())
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
                let result = v.chars().nth(index).map(|c| Value::String(String::from(c)));
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
            Ok(v.entry(index.clone()).or_insert(Value::Nil))
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
            match v.get_mut(&Value::from(key)) {
                None => exception!("field {:?} not found in dict", key),
                Some(v) => Ok(v),
            }
        } else if let Self::Struct(m, v) = self {
            match v.get_mut(&Ustr::from(key)) {
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
                contents.chars().map(|c| Value::String(String::from(c))),
            )),
            _ => None,
        }
    }

    pub fn into_iter(self) -> Option<Box<dyn Iterator<Item = Value>>> {
        match self {
            Self::List(values) => Some(Box::new((*values).clone().into_iter())),
            Self::Dict(pairs) => Some(Box::new(
                pairs
                    .into_iter()
                    .map(|(k, v)| Self::Tuple(vec![k.clone(), v.clone()])),
            )),
            Self::String(contents) => Some(Box::new(
                contents
                    .chars()
                    .collect::<Vec<_>>()
                    .into_iter()
                    .map(|c| Value::String(String::from(c))),
            )),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FunctionLiteral {
    pub body: Rc<Node>,
    pub environment: Rc<RefCell<Environment>>,
}

impl Eq for FunctionLiteral {}

impl PartialEq for FunctionLiteral {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.body, &other.body)
    }
}

impl Hash for FunctionLiteral {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        Rc::as_ptr(&self.body).hash(state)
    }
}

impl Ord for FunctionLiteral {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let self_ptr = Rc::as_ptr(&self.body);
        let other_ptr = Rc::as_ptr(&other.body);
        self_ptr.cmp(&other_ptr)
    }
}

impl PartialOrd for FunctionLiteral {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
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
