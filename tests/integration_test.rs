mod common;
use dang::value::Value;

#[test]
fn test_add() {
    assert_eq!(Value::Integer(10), common::run("5 + 5").unwrap())
}
