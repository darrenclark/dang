use std::collections::{HashMap, HashSet};

use assert_matches::assert_matches;
use dang::{
    ast::NodeKind,
    compiler::variable::VariableAllocation,
    compiler::{CompilationState, Compiler, Input, Phase},
    program::Program,
    value::Value,
};

use ustr::ustr as u;

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
        assert_eq!(node.id.raw_id().1, i as u32);
    }
}

#[test]
fn resolve_function_variables() {
    let compilation_state = run_until(
        Phase::ResolveVariables,
        r#"
        let print_words = fn (words) {
            for word in words {
                println(word)
            }
        }
        "#,
    );

    assert_eq!(compilation_state.errors.len(), 0);

    let words_variable_ref = compilation_state
        .ast()
        .iter()
        .find(|n| matches!(&n.kind, NodeKind::VariableRef { identifier } if identifier.unwrap_identifier() == "words")  )
        .expect("couldn't find 'words'");

    let variable_loc = compilation_state
        .variable_allocations
        .get(&words_variable_ref.id)
        .expect("variable location not populated");

    assert_matches!(variable_loc, VariableAllocation::Local { index: 0 });
}

/*#[test]
fn resolve_for_loop_variables() {
    let compilation_state = run_until(
        Phase::ResolveVariables,
        r#"
        for s in "hello" {
          println(s)
        }
        "#,
    );

    let s_variable_ref = compilation_state
        .ast()
        .iter()
        .find(|n| matches!(&n.kind, NodeKind::VariableRef { identifier } if identifier.unwrap_identifier() == "s")  )
        .expect("couldn't find 's'");

    assert_ma
}*/

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

#[test]
fn struct_info_phase() {
    let compilation_state = run_until(
        Phase::StructInfo,
        r#"
        module Point

        struct {
            x,
            y,
            z: 0
        }
        "#,
    );

    assert_eq!(compilation_state.errors.len(), 0);

    let struct_info = compilation_state
        .struct_info
        .expect("struct_info to be set");

    assert_eq!(struct_info.fields, vec!["x", "y", "z"]);
    assert_eq!(struct_info.required_fields, HashSet::from([u("x"), u("y")]));
    assert_eq!(
        struct_info.default_values,
        HashMap::from([(u("z"), Value::Integer(0))])
    );
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
