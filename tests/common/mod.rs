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
        match common::run($c, "(run)") {
            Ok(_) => {}
            Err(reason) => panic!("{}", reason),
        }
    };
    ($module_name: literal, $c:expr) => {
        match common::run($c, $module_name) {
            Ok(_) => {}
            Err(reason) => panic!("{}", reason),
        }
    };
}
pub(crate) use assert_runs;

macro_rules! assert_raises {
    (($line: literal, $msg: literal), $c:expr) => {
        match common::run($c, "(run)") {
            Ok(_) => panic!("code didn't raise"),
            Err(reason) => {
                if reason.source.as_ref().map(|s| s.line).unwrap_or(0) != $line
                    || reason.message != $msg
                {
                    panic!("EXCEPTION: {}\nEXPECTED: line {}: {}", reason, $line, $msg);
                }
            }
        }
    };
}
pub(crate) use assert_raises;

pub fn run(code: &str, module_name: &str) -> Result<Value, Exception> {
    let mut interpreter = Interpreter::new();

    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("tests/");
    interpreter.compiler.module_search_paths.push(d);

    interpreter.run(Input::SourceCode {
        text: code.to_owned(),
        name: module_name.to_owned(),
    })
}

pub fn parse(code: &str) -> Node {
    let result = dang::dang_parser::parse(code, "(parse)");
    result.unwrap()
}
