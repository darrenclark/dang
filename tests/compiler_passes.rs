use dang::{
    compiler::{CompilationState, Compiler, Input, Phase},
    program::Program,
};

mod common;

#[test]
fn node_ids_pass() {
    let compilation_state = run_until(
        Phase::NodeIds,
        r#"
        for x in [1, 2, 3] { print(x) }
        for x in ["a", "b", "c"] { print(x) }

        var y = 1
        if true {
          var y = 5
          y = y + 1
          assert(y == 6)
        }
        assert(y == 1)

        let curry_add = fn(a) {
            fn(b) {
                a + b
            }
        }
        let add3 = curry_add(3)
        let add5 = curry_add(5)
        assert(add3(1) == 4)
        assert(add5(8) == 13)
        "#,
    );

    for (i, node) in compilation_state.ast().iter().enumerate() {
        assert_eq!(node.id.raw_id(), (0, i as u32));
    }
}

#[test]
fn resolve_variables_pass() {
    let compilation_state = run_until(
        Phase::ResolveVariables,
        r#"
        let fib = fn (n) {
            if n < 2 {
                n
            } else {
                fib(n - 1) + fib(n - 2)
            }
        }

        let double = fn (n) {
            add(n, n)
        }

        let add = fn (a, b) {
            a + b
        }

        let add_curry = fn (a) {
            fn (b) {
                add(a, b)
            }
        }

        let count_positives = fn (nums) {
            var count = 0
            for v in nums {
                if v > 0 {
                    count = count + 1
                }
            }
            count
        }

        let result = fib(double(3))
        //print(result)
        "#,
    );

    assert_eq!(compilation_state.errors.len(), 0);

    /*assert_matches!(
        compilation_state
            .ast()
            .find_by_id(pass.globals["fib"])
            .unwrap()
            .kind,
        NodeKind::Let { .. }
    );

    assert_matches!(
        compilation_state
            .ast()
            .find_by_id(pass.globals["double"])
            .unwrap()
            .kind,
        NodeKind::Let { .. }
    );

    assert_matches!(
        compilation_state
            .ast()
            .find_by_id(pass.globals["add"])
            .unwrap()
            .kind,
        NodeKind::Let { .. }
    );

    for node in compilation_state.ast().iter() {
        if matches!(node.kind, NodeKind::VariableRef { .. }) {
            let definition_id = pass.usages_to_definition.get(&node.id);
            assert!(definition_id.is_some());
            assert!(pass.definitions_to_usages[definition_id.unwrap()].contains(&node.id))
        }
    }*/
}

fn run_until(phase: Phase, text: &str) -> CompilationState {
    let mut program = Program::default();
    Compiler::default()
        .run_until(
            phase,
            Input::SourceCode {
                text: text.to_owned(),
                name: "(test)".to_owned(),
            },
            &mut program,
        )
        .unwrap()
}
