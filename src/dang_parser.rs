use std::rc::Rc;

use pest::{
    error::Error,
    iterators::{Pair, Pairs},
    pratt_parser::{Assoc, Op, PrattParser},
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
            pratt: create_pratt_parser(),
        };
        to_ast.to_source_file_node(pairs)
    })
}

#[allow(clippy::result_large_err)]
pub fn parse_pest_only(input: &str) -> Result<Pairs<Rule>, Error<Rule>> {
    DangParser::parse(Rule::source_file, input)
}

fn create_pratt_parser() -> PrattParser<Rule> {
    PrattParser::new()
        .op(Op::infix(Rule::add, Assoc::Left) | Op::infix(Rule::sub, Assoc::Left))
        .op(Op::infix(Rule::mul, Assoc::Left) | Op::infix(Rule::div, Assoc::Left))
        .op(Op::prefix(Rule::neg))
}

struct ToAst {
    source_file_name: Rc<String>,
    pratt: PrattParser<Rule>,
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
            Rule::expr => {
                self.pratt
                    .map_primary(|primary| self.to_ast(primary))
                    .map_prefix(|op, rhs| match op.as_rule() {
                        Rule::neg => self.new_node(
                            op.line_col(),
                            NodeKind::Negate {
                                rhs: Box::new(rhs.unwrap()),
                            },
                        ),
                        _ => unreachable!(),
                    })
                    .map_infix(|lhs, op, rhs| match op.as_rule() {
                        Rule::add => self.new_node(
                            op.line_col(),
                            NodeKind::Add {
                                lhs: Box::new(lhs.unwrap()),
                                rhs: Box::new(rhs.unwrap()),
                            },
                        ),
                        Rule::sub => self.new_node(
                            op.line_col(),
                            NodeKind::Sub {
                                lhs: Box::new(lhs.unwrap()),
                                rhs: Box::new(rhs.unwrap()),
                            },
                        ),
                        Rule::mul => self.new_node(
                            op.line_col(),
                            NodeKind::Mul {
                                lhs: Box::new(lhs.unwrap()),
                                rhs: Box::new(rhs.unwrap()),
                            },
                        ),
                        Rule::div => self.new_node(
                            op.line_col(),
                            NodeKind::Div {
                                lhs: Box::new(lhs.unwrap()),
                                rhs: Box::new(rhs.unwrap()),
                            },
                        ),
                        //Rule::pow => (1..rhs + 1).map(|_| lhs).product(),
                        _ => unreachable!(),
                    })
                    .parse(pair.into_inner())
            }
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
            _ => todo!(),
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
