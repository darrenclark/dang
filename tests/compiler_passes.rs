use common::parse;
use dang::{compiler::node_ids_pass::NodeIdsPass, module::ModuleId};

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
