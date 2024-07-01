use common::assert_runs;

mod common;

#[test]
fn test_add() {
    assert_runs! {
        "
        let f = fn(x, y) {
            (x + 1) * y
        }

        assert(f(1, 2) == 4)
        "
    };
}
