use crate::ast::{Node, Source};
use std::fmt;

#[derive(Debug, Clone)]
pub struct Exception {
    pub source: Option<Source>,
    pub message: String,
}

impl Exception {
    pub fn new(message: String) -> Exception {
        Exception {
            source: None,
            message,
        }
    }

    pub fn at_node(node: &Node, message: &str) -> Exception {
        Exception {
            source: Some(node.source.clone()),
            message: message.to_owned(),
        }
    }

    pub fn set_source_from_node(&mut self, node: &Node) {
        self.source = Some(node.source.clone());
    }
}

impl fmt::Display for Exception {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.source {
            Some(source) => write!(
                f,
                "exception at {}:{}:{}: {}",
                source.file, source.start.line, source.start.col, self.message
            ),
            None => write!(f, "exception: {}", self.message),
        }
    }
}

macro_rules! exception {
    ($($arg:tt)*) => {
        return Err(Exception::new(format!($($arg)*)))
    };
}

pub(crate) use exception;
