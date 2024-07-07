use assert_matches::assert_matches;
use common::parse;
use dang::{
    ast::NodeKind,
    compiler::{node_ids_pass::NodeIdsPass, resolve_variables::ResolveVariablesPass},
    module::ModuleId,
};

mod common;

#[test]
fn node_ids_pass() {
    let mut ast = parse(
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

    NodeIdsPass::new(ModuleId::from_raw(0)).run(&mut ast);

    for (i, node) in ast.iter().enumerate() {
        assert_eq!(node.id.raw_id(), (0, i as u32));
    }
}

#[test]
fn resolve_variables_pass() {
    let mut ast = parse(
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

    NodeIdsPass::new(ModuleId::from_raw(0)).run(&mut ast);

    let mut pass = ResolveVariablesPass::new();
    pass.run(&ast);

    assert_eq!(pass.errors.len(), 0);

    assert_matches!(
        ast.find_by_id(pass.globals["fib"]).unwrap().kind,
        NodeKind::Let { .. }
    );

    assert_matches!(
        ast.find_by_id(pass.globals["double"]).unwrap().kind,
        NodeKind::Let { .. }
    );

    assert_matches!(
        ast.find_by_id(pass.globals["add"]).unwrap().kind,
        NodeKind::Let { .. }
    );

    for node in ast.iter() {
        if matches!(node.kind, NodeKind::VariableRef { .. }) {
            let definition_id = pass.usages_to_definition.get(&node.id);
            assert!(definition_id.is_some());
            assert!(pass.definitions_to_usages[definition_id.unwrap()].contains(&node.id))
        }
    }
}
