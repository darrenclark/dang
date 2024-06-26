use pest::{
    error::Error,
    iterators::{Pair, Pairs},
    Parser,
};
use pest_derive::Parser;

use crate::ast::{Node, NodeKind};

#[derive(Parser)]
#[grammar = "dang.pest"]
pub struct DangParser;

#[allow(clippy::result_large_err)]
pub fn parse(input: &str) -> Result<Node, Error<Rule>> {
    let result = DangParser::parse(Rule::source_file, input);
    result.map(|pairs| Node {
        kind: NodeKind::SourceFile(to_asts(pairs)),
    })
}

fn to_asts(pairs: Pairs<Rule>) -> Vec<Node> {
    pairs.filter_map(to_ast).collect()
}

fn to_ast(pair: Pair<Rule>) -> Option<Node> {
    match pair.as_rule() {
        Rule::EOI => None,
        Rule::WHITESPACE => panic!(),
        Rule::source_file => panic!(),
        Rule::identifier => new_node(NodeKind::Identifier(String::from(pair.as_str()))),
        Rule::stmt => panic!(),
        Rule::let_stmt => {
            let mut iter = pair.into_inner();
            let identifier = to_ast(iter.next().unwrap()).unwrap();
            let expr = to_ast(iter.next().unwrap()).unwrap();
            new_node(NodeKind::Let {
                identifier: Box::new(identifier),
                expr: Box::new(expr),
            })
        }
        Rule::var_stmt => {
            let mut iter = pair.into_inner();
            let identifier = to_ast(iter.next().unwrap()).unwrap();
            let expr = to_ast(iter.next().unwrap()).unwrap();
            new_node(NodeKind::Var {
                identifier: Box::new(identifier),
                expr: Box::new(expr),
            })
        }
        Rule::assignment => {
            let mut iter = pair.into_inner();
            let identifier = to_ast(iter.next().unwrap()).unwrap();
            let expr = to_ast(iter.next().unwrap()).unwrap();
            new_node(NodeKind::Assignment {
                identifier: Box::new(identifier),
                expr: Box::new(expr),
            })
        }
        Rule::expr => panic!(),
        Rule::function_call => {
            let mut iter = pair.into_inner();
            let func = to_ast(iter.next().unwrap()).unwrap();

            let args: Vec<Node> = iter
                .next()
                .unwrap()
                .into_inner()
                .filter_map(to_ast)
                .collect();

            new_node(NodeKind::FunctionCall {
                function: Box::new(func),
                args,
            })
        }
        Rule::args => panic!(),
        Rule::string_literal => {
            let string = String::from(pair.into_inner().next().unwrap().as_str());
            new_node(NodeKind::StringLiteral(string))
        }
        Rule::string_contents => panic!(),
        Rule::char => todo!(),
        Rule::number_literal => {
            // TODO: Floats
            new_node(NodeKind::IntegerLiteral(
                pair.as_str().parse::<i64>().unwrap(),
            ))
        }
    }
}

fn new_node(kind: NodeKind) -> Option<Node> {
    Some(Node { kind })
}
