use std::rc::Rc;

use pest::{
    error::Error,
    iterators::{Pair, Pairs},
    pratt_parser::{Assoc, Op, PrattParser},
    Parser,
};
use pest_derive::Parser;

use crate::ast::{BinOp, Node, NodeKind, Source, UnaryOp};

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
        .op(Op::infix(Rule::logical_or, Assoc::Left))
        .op(Op::infix(Rule::logical_and, Assoc::Left))
        .op(Op::infix(Rule::eq, Assoc::Left) | Op::infix(Rule::neq, Assoc::Left))
        .op(Op::infix(Rule::gt, Assoc::Left)
            | Op::infix(Rule::gte, Assoc::Left)
            | Op::infix(Rule::lte, Assoc::Left)
            | Op::infix(Rule::lte, Assoc::Left))
        .op(Op::infix(Rule::add, Assoc::Left) | Op::infix(Rule::sub, Assoc::Left))
        .op(Op::infix(Rule::mul, Assoc::Left) | Op::infix(Rule::div, Assoc::Left))
        .op(Op::prefix(Rule::neg) | Op::prefix(Rule::logical_neg))
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
            Rule::body => {
                let body: Vec<Node> = pair.into_inner().filter_map(|p| self.to_ast(p)).collect();
                self.new_node(line_col, NodeKind::Body(body))
            }
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
            Rule::if_stmt => {
                let mut iter = pair.into_inner();
                let condition = self.to_ast(iter.next().unwrap()).unwrap();
                let body = self.to_ast(iter.next().unwrap()).unwrap();
                let else_branch = iter.next().map(|p| self.to_ast(p).unwrap());
                self.new_node(
                    line_col,
                    NodeKind::If {
                        condition: Box::new(condition),
                        body: Box::new(body),
                        else_branch: else_branch.map(Box::new),
                    },
                )
            }
            Rule::expr => {
                self.pratt
                    .map_primary(|primary| self.to_ast(primary))
                    .map_prefix(|op, rhs| match op.as_rule() {
                        Rule::logical_neg => self.new_node(
                            op.line_col(),
                            NodeKind::UnaryOp {
                                op: UnaryOp::LogicalNeg,
                                rhs: Box::new(rhs.unwrap()),
                            },
                        ),
                        Rule::neg => self.new_node(
                            op.line_col(),
                            NodeKind::UnaryOp {
                                op: UnaryOp::Neg,
                                rhs: Box::new(rhs.unwrap()),
                            },
                        ),
                        _ => unreachable!(),
                    })
                    .map_infix(|lhs, op, rhs| match op.as_rule() {
                        Rule::logical_or => self.new_node(
                            op.line_col(),
                            NodeKind::BinaryOp {
                                op: BinOp::LogicalOr,
                                lhs: Box::new(lhs.unwrap()),
                                rhs: Box::new(rhs.unwrap()),
                            },
                        ),
                        Rule::logical_and => self.new_node(
                            op.line_col(),
                            NodeKind::BinaryOp {
                                op: BinOp::LogicalAnd,
                                lhs: Box::new(lhs.unwrap()),
                                rhs: Box::new(rhs.unwrap()),
                            },
                        ),
                        Rule::eq => self.new_node(
                            op.line_col(),
                            NodeKind::BinaryOp {
                                op: BinOp::Eq,
                                lhs: Box::new(lhs.unwrap()),
                                rhs: Box::new(rhs.unwrap()),
                            },
                        ),
                        Rule::neq => self.new_node(
                            op.line_col(),
                            NodeKind::BinaryOp {
                                op: BinOp::Neq,
                                lhs: Box::new(lhs.unwrap()),
                                rhs: Box::new(rhs.unwrap()),
                            },
                        ),
                        Rule::gt => self.new_node(
                            op.line_col(),
                            NodeKind::BinaryOp {
                                op: BinOp::Gt,
                                lhs: Box::new(lhs.unwrap()),
                                rhs: Box::new(rhs.unwrap()),
                            },
                        ),
                        Rule::gte => self.new_node(
                            op.line_col(),
                            NodeKind::BinaryOp {
                                op: BinOp::Gte,
                                lhs: Box::new(lhs.unwrap()),
                                rhs: Box::new(rhs.unwrap()),
                            },
                        ),
                        Rule::lt => self.new_node(
                            op.line_col(),
                            NodeKind::BinaryOp {
                                op: BinOp::Lt,
                                lhs: Box::new(lhs.unwrap()),
                                rhs: Box::new(rhs.unwrap()),
                            },
                        ),
                        Rule::lte => self.new_node(
                            op.line_col(),
                            NodeKind::BinaryOp {
                                op: BinOp::Lte,
                                lhs: Box::new(lhs.unwrap()),
                                rhs: Box::new(rhs.unwrap()),
                            },
                        ),
                        Rule::add => self.new_node(
                            op.line_col(),
                            NodeKind::BinaryOp {
                                op: BinOp::Add,
                                lhs: Box::new(lhs.unwrap()),
                                rhs: Box::new(rhs.unwrap()),
                            },
                        ),
                        Rule::sub => self.new_node(
                            op.line_col(),
                            NodeKind::BinaryOp {
                                op: BinOp::Sub,
                                lhs: Box::new(lhs.unwrap()),
                                rhs: Box::new(rhs.unwrap()),
                            },
                        ),
                        Rule::mul => self.new_node(
                            op.line_col(),
                            NodeKind::BinaryOp {
                                op: BinOp::Mul,
                                lhs: Box::new(lhs.unwrap()),
                                rhs: Box::new(rhs.unwrap()),
                            },
                        ),
                        Rule::div => self.new_node(
                            op.line_col(),
                            NodeKind::BinaryOp {
                                op: BinOp::Div,
                                lhs: Box::new(lhs.unwrap()),
                                rhs: Box::new(rhs.unwrap()),
                            },
                        ),
                        //Rule::pow => (1..rhs + 1).map(|_| lhs).product(),
                        _ => unreachable!(),
                    })
                    .parse(pair.into_inner())
            }
            Rule::function_literal => {
                let mut iter = pair.into_inner();
                let arg_names: Vec<Node> = iter
                    .next()
                    .unwrap()
                    .into_inner()
                    .filter_map(|p| self.to_ast(p))
                    .collect();

                let body: Vec<Node> = iter.filter_map(|p| self.to_ast(p)).collect();

                self.new_node(line_col, NodeKind::FunctionLiteral { arg_names, body })
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
            Rule::list_literal => {
                let elements: Vec<Node> =
                    pair.into_inner().filter_map(|p| self.to_ast(p)).collect();
                self.new_node(line_col, NodeKind::ListLiteral(elements))
            }
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
            Rule::bool_literal => self.new_node(
                line_col,
                NodeKind::BoolLiteral(pair.as_str().parse::<bool>().unwrap()),
            ),
            Rule::nil_literal => self.new_node(line_col, NodeKind::NilLiteral),
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
