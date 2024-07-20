#![allow(unused_macros)]
#![allow(unused_imports)]
#![allow(dead_code)]

use std::path::PathBuf;

use dang::ast::Node;
use dang::compiler::Input;
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

    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("tests/");
    interpreter.compiler.module_search_paths.push(d);

    let result = interpreter.run(Input::SourceCode {
        text: code.to_owned(),
        name: "(run)".to_owned(),
    });
    if result.is_ok() {
        result
    } else {
        panic!("{}", result.unwrap_err());
    }
}

pub fn parse(code: &str) -> Node {
    let result = dang::dang_parser::parse(code, "(parse)");
    result.unwrap()
}
