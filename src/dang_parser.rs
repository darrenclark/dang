use std::rc::Rc;

use pest::{
    error::Error,
    iterators::{Pair, Pairs},
    Parser,
};
use pest_derive::Parser;

use crate::ast::{Node, NodeKind, Source};

#[derive(Parser)]
#[grammar = "dang.pest"]
pub struct DangParser;

#[allow(clippy::result_large_err)]
pub fn parse(input: &str, source_file_name: &str) -> Result<Node, Error<Rule>> {
    let result = DangParser::parse(Rule::source_file, input);
    result.map(|pairs| {
        let to_ast = ToAst {
            source_file_name: Rc::new(String::from(source_file_name)),
        };
        to_ast.to_source_file_node(pairs)
    })
}

struct ToAst {
    source_file_name: Rc<String>,
}

impl ToAst {
    fn to_source_file_node(&self, pairs: Pairs<Rule>) -> Node {
        self.new_node((0, 0), NodeKind::SourceFile(self.to_asts(pairs)))
            .unwrap()
    }

    fn to_asts(&self, pairs: Pairs<Rule>) -> Vec<Node> {
        pairs.filter_map(|p| self.to_ast(p)).collect()
    }

    fn to_ast(&self, pair: Pair<Rule>) -> Option<Node> {
        let line_col = pair.line_col();
        match pair.as_rule() {
            Rule::EOI => None,
            Rule::WHITESPACE => panic!(),
            Rule::source_file => panic!(),
            Rule::identifier => {
                self.new_node(line_col, NodeKind::Identifier(String::from(pair.as_str())))
            }
            Rule::stmt => panic!(),
            Rule::let_stmt => {
                let mut iter = pair.into_inner();
                let identifier = self.to_ast(iter.next().unwrap()).unwrap();
                let expr = self.to_ast(iter.next().unwrap()).unwrap();
                self.new_node(
                    line_col,
                    NodeKind::Let {
                        identifier: Box::new(identifier),
                        expr: Box::new(expr),
                    },
                )
            }
            Rule::var_stmt => {
                let mut iter = pair.into_inner();
                let identifier = self.to_ast(iter.next().unwrap()).unwrap();
                let expr = self.to_ast(iter.next().unwrap()).unwrap();
                self.new_node(
                    line_col,
                    NodeKind::Var {
                        identifier: Box::new(identifier),
                        expr: Box::new(expr),
                    },
                )
            }
            Rule::assignment => {
                let mut iter = pair.into_inner();
                let identifier = self.to_ast(iter.next().unwrap()).unwrap();
                let expr = self.to_ast(iter.next().unwrap()).unwrap();
                self.new_node(
                    line_col,
                    NodeKind::Assignment {
                        identifier: Box::new(identifier),
                        expr: Box::new(expr),
                    },
                )
            }
            Rule::expr => panic!(),
            Rule::function_call => {
                let mut iter = pair.into_inner();
                let func = self.to_ast(iter.next().unwrap()).unwrap();

                let args: Vec<Node> = iter
                    .next()
                    .unwrap()
                    .into_inner()
                    .filter_map(|p| self.to_ast(p))
                    .collect();

                self.new_node(
                    line_col,
                    NodeKind::FunctionCall {
                        function: Box::new(func),
                        args,
                    },
                )
            }
            Rule::args => panic!(),
            Rule::string_literal => {
                let string = String::from(pair.into_inner().next().unwrap().as_str());
                self.new_node(line_col, NodeKind::StringLiteral(string))
            }
            Rule::string_contents => panic!(),
            Rule::char => todo!(),
            Rule::number_literal => {
                // TODO: Floats
                self.new_node(
                    line_col,
                    NodeKind::IntegerLiteral(pair.as_str().parse::<i64>().unwrap()),
                )
            }
        }
    }

    fn new_node(&self, line_col: (usize, usize), kind: NodeKind) -> Option<Node> {
        let node = Node {
            kind,
            source: Source {
                file: self.source_file_name.clone(),
                line: line_col.0,
                col: line_col.1,
            },
        };
        Some(node)
    }
}
