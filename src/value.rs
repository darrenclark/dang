use std::rc::Rc;

use crate::{
    ast::Node,
    interpreter::{exception, Exception},
};

#[derive(Debug, Clone, PartialOrd, PartialEq)]
pub enum Value {
    Nil,
    Bool(bool),
    String(String),
    Integer(i64),
    NativeFunc(fn(&[Value]) -> Result<Value, Exception>),
    Func(FunctionLiteralRc),
    List(Vec<Value>),
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
        Self::List(values)
    }
}

impl Value {
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
}

#[derive(Debug, Clone)]
pub struct FunctionLiteralRc(pub Rc<Node>);

impl PartialEq for FunctionLiteralRc {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl PartialOrd for FunctionLiteralRc {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self == other {
            Some(std::cmp::Ordering::Equal)
        } else {
            None
        }
    }
}
