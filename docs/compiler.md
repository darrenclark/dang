# Compiler

- Interpreter
    - uses data from Program (mainly AST)
- Program
    - stores "fully compiled" modules
- Compiler
    - CompilationState
    - Pass(s)
        - mutable ref to Program
        - mutable ref to CompilationState

```
fn compile(&self, module_name: &str, program: &mut Program) -> Result<(), Vec<Exception>> {
    if program.module_exists(module_name) {
        return Ok(())
    }

    let mut state = CompilationState::new(module_name.to_owned());

    for pass in self.passes {
        pass.new_impl(self, program, &mut state).run();
    }
}

// for test cases
fn compile_until(&self, module_name: &str, program: &mut Program, until_phase: Phase) -> Result<CompilationState, Vec<Exception>> {
    // ...
}
```
