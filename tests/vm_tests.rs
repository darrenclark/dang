use dang::{
    value::Value,
    vm::{chunk::Chunk, inst::Instr, VM},
};

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

    let mut vm = VM::new(chunk);

    assert_eq!(vm.run().unwrap(), Value::Integer(5));
}
