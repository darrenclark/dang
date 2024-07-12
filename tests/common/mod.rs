#![allow(unused_macros)]
#![allow(unused_imports)]
#![allow(dead_code)]

use dang::ast::Node;
use dang::interpreter::Exception;
use dang::interpreter::Interpreter;
use dang::value::Value;

macro_rules! assert_runs {
    ($c:expr) => {
        match common::run($c) {
            Ok(_) => {}
            Err(reason) => panic!("{}", reason),
        }
    };
}
pub(crate) use assert_runs;

pub fn run(code: &str) -> Result<Value, Exception> {
    let mut interpreter = Interpreter::new();
    let result = dang::dang_parser::parse(code, "(run)");
    if let Ok(pairs) = result {
        interpreter.eval(&pairs)
    } else {
        panic!("{}", result.unwrap_err());
    }
}

pub fn parse(code: &str) -> Node {
    let result = dang::dang_parser::parse(code, "(parse)");
    result.unwrap()
}
