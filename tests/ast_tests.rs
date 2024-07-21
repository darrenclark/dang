use assert_matches::assert_matches;
use dang::ast::NodeKind;

mod common;

#[test]
fn iterating_struct_def() {
    let parsed = common::parse("struct { x, y: 0, z }");
    let struct_def = match &parsed.kind {
        NodeKind::SourceFile(children) => children[0].clone(),
        _ => unreachable!(),
    };

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
    let parsed = common::parse("Point{x: 0, y: 0}");
    let struct_literal = match &parsed.kind {
        NodeKind::SourceFile(children) => children[0].clone(),
        _ => unreachable!(),
    };

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
