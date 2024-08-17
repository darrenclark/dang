use std::collections::{HashMap, HashSet};

use dang::{
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
