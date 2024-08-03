use crate::{interpreter::Exception, program::Program, value::Value, vm::inst::Instr};

use super::{CompilationState, Compiler};

pub fn emit_bytecode_phase(
    _compiler: &Compiler,
    compilation_state: &mut CompilationState,
    _program: &mut Program,
) -> Result<(), Exception> {
    let chunk = &mut compilation_state.chunk;

    let module = chunk.write_constant(Value::Symbol(compilation_state.module_name.0));
    let test_var = chunk.write_constant(Value::Symbol("x".into()));

    // let x = 5
    let five = chunk.write_constant(Value::Integer(5));
    chunk.write(Instr::constant(five));
    chunk.write(Instr::set_global(module, test_var));

    // x
    chunk.write(Instr::get_global(module, test_var));
    chunk.write(Instr::return_());

    Ok(())
}
