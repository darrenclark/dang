use common::assert_runs_vm;
use dang::{
    value::Value,
    vm::{chunk::Chunk, function::Function, inst::Instr, VM},
};

mod common;

#[test]
fn example() {
    let mut chunk = Chunk::new();

    let module = chunk.write_constant(Value::Symbol("example".into()));
    let test_var = chunk.write_constant(Value::Symbol("test_var".into()));

    // test_var = 5
    let five = chunk.write_constant(Value::Integer(5));
    chunk.write(Instr::constant(five));
    chunk.write(Instr::set_global(module, test_var));

    // test_var
    chunk.write(Instr::get_global(module, test_var));
    chunk.write(Instr::return_());

    let mut vm = VM::new(Function::new(chunk));

    assert_eq!(vm.run().unwrap(), Value::Integer(5));
}

#[test]
fn simple_math() {
    let val = assert_runs_vm! {
        r#"
        let a = 3
        let b = 5
        let c = 40

        var d = 0
        d = d + c
        d = d + 5

        let res = d / (a * -b)
        res
        "#
    };

    assert_eq!(val, Value::Integer(-3));
}

#[test]
fn short_circuiting() {
    assert_runs_vm! {
        r#"
        __builtin raise

        let a = true
        let b = "true"

        (a && b) || raise("This should not be called")
        "#
    };
}

#[test]
fn if_statements() {
    assert_runs_vm! {
        r#"
        __builtin raise

        let a = 5
        let b = 3
        if a > b {
            "ok"
        } else {
            raise("This should not be called")
        }
        "#
    };
}
