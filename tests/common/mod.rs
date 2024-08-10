#![allow(unused_macros)]
#![allow(unused_imports)]
#![allow(dead_code)]

use std::path::PathBuf;

use dang::ast::Node;
use dang::compiler::Compiler;
use dang::compiler::Input;
use dang::interpreter::Exception;
use dang::interpreter::Interpreter;
use dang::program::Program;
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

macro_rules! assert_runs_vm {
    ($c:expr) => {
        match common::run_vm($c, "(run)") {
            Ok(val) => val,
            Err(reason) => panic!("{}", reason),
        }
    };
    ($module_name: literal, $c:expr) => {
        match common::run_vm($c, $module_name) {
            Ok(val) => val,
            Err(reason) => panic!("{}", reason),
        }
    };
}
pub(crate) use assert_runs_vm;

macro_rules! assert_runs_both {
    ($c:expr) => {
        assert_runs_vm!($c);
    };
    ($module_name: literal, $c:expr) => {
        assert_runs_vm!($module_name, $c);
    };
}
pub(crate) use assert_runs_both;

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
use dang::vm::VM;

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

pub fn run_vm(code: &str, module_name: &str) -> Result<Value, Exception> {
    let mut compiler = Compiler::for_vm();
    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("tests/");
    compiler.module_search_paths.push(d);

    let mut program = Program::default();

    let input = Input::SourceCode {
        text: code.to_owned(),
        name: module_name.to_owned(),
    };

    match compiler.compile(input, &mut program) {
        Ok(_) => {
            let module_name = program.module_names().last().cloned().unwrap();
            let module = program.get_module(&module_name).unwrap();
            let mut vm = VM::new(module.function.clone());
            vm.run()
        }
        Err(e) => {
            panic!("failed to compile: {:?}", e);
        }
    }
}

pub fn parse(code: &str) -> Node {
    let result = dang::dang_parser::parse(code, "(parse)");
    result.unwrap()
}
