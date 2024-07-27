use std::{collections::BTreeMap, rc::Rc};

use assert_matches::assert_matches;
use dang::{
    ast::{Node, NodeKind},
    value::Value,
};

mod common;

pub fn parse(code: &str) -> Node {
    let result = common::parse(code);
    match result.kind {
        NodeKind::SourceFile(children) => children[0].clone(),
        _ => unreachable!(),
    }
}

#[test]
fn iterating_struct_def() {
    let struct_def = parse("struct { x, y: 0, z }");

    let mut iter = struct_def.iter();
    assert_matches!(&iter.next().unwrap().kind, NodeKind::StructDef { .. });
    assert_matches!(&iter.next().unwrap().kind, NodeKind::Identifier(ident) if ident == "x");
    assert_matches!(&iter.next().unwrap().kind, NodeKind::Identifier(ident) if ident == "y");
    assert_matches!(&iter.next().unwrap().kind, NodeKind::IntegerLiteral(0));
    assert_matches!(&iter.next().unwrap().kind, NodeKind::Identifier(ident) if ident == "z");
    assert_matches!(&iter.next(), None);
}

#[test]
fn iterating_struct_literal() {
    let struct_literal = parse("Point{x: 0, y: 0}");

    let mut iter = struct_literal.iter();
    assert_matches!(&iter.next().unwrap().kind, NodeKind::StructLiteral { .. });
    assert_matches!(&iter.next().unwrap().kind, NodeKind::VariableRef { .. });
    assert_matches!(&iter.next().unwrap().kind, NodeKind::Identifier(ident) if ident == "Point");
    assert_matches!(&iter.next().unwrap().kind, NodeKind::Identifier(ident) if ident == "x");
    assert_matches!(&iter.next().unwrap().kind, NodeKind::IntegerLiteral(0));
    assert_matches!(&iter.next().unwrap().kind, NodeKind::Identifier(ident) if ident == "y");
    assert_matches!(&iter.next().unwrap().kind, NodeKind::IntegerLiteral(0));
    assert_matches!(&iter.next(), None);
}

#[test]
fn compile_time_values() {
    assert_eq!(parse("5").compile_time_value(), Some(Value::Integer(5)));

    assert_eq!(
        parse("\"hi\"").compile_time_value(),
        Some(Value::String("hi".to_owned()))
    );

    assert_eq!(parse("true").compile_time_value(), Some(Value::Bool(true)));

    assert_eq!(parse("nil").compile_time_value(), Some(Value::Nil));

    let expected_list = Value::List(Rc::new(vec![
        Value::Integer(1),
        Value::Integer(2),
        Value::Integer(3),
    ]));
    assert_eq!(parse("[1, 2, 3]").compile_time_value(), Some(expected_list));

    let expected_dict = Value::Dict(BTreeMap::from([
        (Value::String("x".to_owned()), Value::Integer(1)),
        (Value::String("y".to_owned()), Value::Integer(2)),
    ]));
    assert_eq!(
        parse("{x: 1, y: 2}").compile_time_value(),
        Some(expected_dict)
    );

    assert!(parse("add(1, 1)").compile_time_value().is_none());
    assert!(parse("[one()]").compile_time_value().is_none());
}
