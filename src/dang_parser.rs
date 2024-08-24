use std::rc::Rc;

use pest::{
    error::Error,
    iterators::{Pair, Pairs},
    pratt_parser::{Assoc, Op, PrattParser},
    Parser,
};
use pest_derive::Parser;
use unescape::unescape;

use crate::{
    ast::{BinOp, ImportKind, Node, NodeId, NodeKind, Source, UnaryOp},
    line_col::LineCol,
};

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
            | Op::infix(Rule::lt, Assoc::Left)
            | Op::infix(Rule::lte, Assoc::Left))
        .op(Op::infix(Rule::add, Assoc::Left) | Op::infix(Rule::sub, Assoc::Left))
        .op(Op::infix(Rule::mul, Assoc::Left) | Op::infix(Rule::div, Assoc::Left))
        .op(Op::infix(Rule::pipe, Assoc::Left))
        .op(Op::prefix(Rule::neg) | Op::prefix(Rule::logical_neg))
}

struct ToAst {
    source_file_name: Rc<String>,
    pratt: PrattParser<Rule>,
}

impl ToAst {
    fn to_source_file_node(&self, pairs: Pairs<Rule>) -> Node {
        let loc = (LineCol::unknown(), LineCol::unknown());
        self.new_node(loc, NodeKind::SourceFile(self.to_asts(pairs)))
            .unwrap()
    }

    fn to_asts(&self, pairs: Pairs<Rule>) -> Vec<Node> {
        pairs.filter_map(|p| self.to_ast(p)).collect()
    }

    fn to_ast(&self, pair: Pair<Rule>) -> Option<Node> {
        let loc = self.location(&pair);
        match pair.as_rule() {
            Rule::EOI => None,
            Rule::WHITESPACE => panic!(),
            Rule::source_file => panic!(),
            Rule::module => {
                let module_name = pair.into_inner().next().unwrap().as_str().to_owned();
                self.new_node(loc, NodeKind::Module(module_name))
            }
            Rule::import => {
                let mut iter = pair.into_inner().next().unwrap().into_inner();
                let module_name = iter.next().unwrap().as_str().to_string();

                let next = iter.next();
                let import_kind = if let Some(next) = next {
                    match next.as_rule() {
                        Rule::identifier => ImportKind::Field {
                            module_name,
                            field_name: next.as_str().to_owned(),
                        },
                        Rule::import_all => ImportKind::AllFields { module_name },
                        _ => unreachable!(),
                    }
                } else {
                    ImportKind::Module { module_name }
                };

                self.new_node(loc, NodeKind::Import(import_kind))
            }
            Rule::struct_def => {
                let fields: Vec<(Node, Option<Node>)> = pair
                    .into_inner()
                    .map(|p| {
                        let mut iter = p.into_inner();
                        let name = self.to_ast(iter.next().unwrap()).unwrap();
                        let default_value = iter.next().and_then(|p| self.to_ast(p));
                        (name, default_value)
                    })
                    .collect();
                self.new_node(loc, NodeKind::StructDef { fields })
            }
            Rule::body => {
                let body: Vec<Node> = pair.into_inner().filter_map(|p| self.to_ast(p)).collect();
                self.new_node(loc, NodeKind::Body(body))
            }
            Rule::identifier => {
                self.new_node(loc, NodeKind::Identifier(String::from(pair.as_str())))
            }
            Rule::stmt => panic!(),
            Rule::builtin_stmt => {
                let mut iter = pair.into_inner();
                let identifier = self.to_ast(iter.next().unwrap()).unwrap();

                self.new_node(
                    loc,
                    NodeKind::Builtin {
                        identifier: Box::new(identifier),
                    },
                )
            }
            Rule::let_stmt => {
                let mut iter = pair.into_inner();
                let pattern = self.to_ast(iter.next().unwrap()).unwrap();
                let expr = self.to_ast(iter.next().unwrap()).unwrap();
                self.new_node(
                    loc,
                    NodeKind::Let {
                        pattern: Box::new(pattern),
                        expr: Box::new(expr),
                    },
                )
            }
            Rule::var_stmt => {
                let mut iter = pair.into_inner();
                let pattern = self.to_ast(iter.next().unwrap()).unwrap();
                let expr = self.to_ast(iter.next().unwrap()).unwrap();
                self.new_node(
                    loc,
                    NodeKind::Var {
                        pattern: Box::new(pattern),
                        expr: Box::new(expr),
                    },
                )
            }
            Rule::assignment => {
                let mut iter = pair.into_inner();
                let pattern = self.to_ast(iter.next().unwrap()).unwrap();
                let expr = self.to_ast(iter.next().unwrap()).unwrap();
                self.new_node(
                    loc,
                    NodeKind::Assignment {
                        pattern: Box::new(pattern),
                        expr: Box::new(expr),
                    },
                )
            }
            Rule::field_assignment => {
                let mut iter = pair.into_inner();

                let mut path_iter = iter.next().unwrap().into_inner();
                let object = self.to_ast(path_iter.next().unwrap()).unwrap();
                let path = path_iter
                    .map(|p| {
                        let loc = self.location(&p);
                        let rule = p.as_rule();
                        let expr = self.to_ast(p.into_inner().next().unwrap()).unwrap();
                        match rule {
                            Rule::subscript => self
                                .new_node(
                                    loc,
                                    NodeKind::FieldAssignmentPathSubscript {
                                        key: Box::new(expr),
                                    },
                                )
                                .unwrap(),
                            Rule::field_access => self
                                .new_node(
                                    loc,
                                    NodeKind::FieldAssignmentPathField {
                                        key: Box::new(expr),
                                    },
                                )
                                .unwrap(),
                            _ => unreachable!(),
                        }
                    })
                    .collect::<Vec<_>>();

                let expr = self.to_ast(iter.next().unwrap()).unwrap();

                self.new_node(
                    loc,
                    NodeKind::FieldAssignment {
                        variable_ref: Box::new(object),
                        path,
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
                    loc,
                    NodeKind::If {
                        condition: Box::new(condition),
                        body: Box::new(body),
                        else_branch: else_branch.map(Box::new),
                    },
                )
            }
            Rule::for_stmt => {
                let mut iter = pair.into_inner();
                let pattern = self.to_ast(iter.next().unwrap()).unwrap();
                let enumerable = self.to_ast(iter.next().unwrap()).unwrap();
                let body = self.to_ast(iter.next().unwrap()).unwrap();
                self.new_node(
                    loc,
                    NodeKind::For {
                        pattern: Box::new(pattern),
                        enumerable: Box::new(enumerable),
                        body: Box::new(body),
                    },
                )
            }
            Rule::pattern => {
                let child = pair.into_inner().next().unwrap();
                self.to_ast(child)
            }
            Rule::pattern_identifier => {
                let identifier = self.to_ast(pair.into_inner().next().unwrap()).unwrap();
                if identifier.unwrap_identifier() == "_" {
                    self.new_node(loc, NodeKind::PatternWildcard)
                } else {
                    self.new_node(
                        loc,
                        NodeKind::PatternIdentifier {
                            identifier: Box::new(identifier),
                        },
                    )
                }
            }
            Rule::pattern_tuple => {
                let elements: Vec<Node> =
                    pair.into_inner().filter_map(|p| self.to_ast(p)).collect();
                self.new_node(loc, NodeKind::PatternTuple { elements })
            }
            Rule::expr => {
                self.pratt
                    .map_primary(|primary| self.to_ast(primary))
                    .map_prefix(|op, rhs| match op.as_rule() {
                        Rule::logical_neg => self.new_node(
                            self.location(&op),
                            NodeKind::UnaryOp {
                                op: UnaryOp::LogicalNeg,
                                rhs: Box::new(rhs.unwrap()),
                            },
                        ),
                        Rule::neg => self.new_node(
                            self.location(&op),
                            NodeKind::UnaryOp {
                                op: UnaryOp::Neg,
                                rhs: Box::new(rhs.unwrap()),
                            },
                        ),
                        _ => unreachable!(),
                    })
                    .map_infix(|lhs, op, rhs| match op.as_rule() {
                        Rule::logical_or => self.new_node(
                            self.location(&op),
                            NodeKind::BinaryOp {
                                op: BinOp::LogicalOr,
                                lhs: Box::new(lhs.unwrap()),
                                rhs: Box::new(rhs.unwrap()),
                            },
                        ),
                        Rule::logical_and => self.new_node(
                            self.location(&op),
                            NodeKind::BinaryOp {
                                op: BinOp::LogicalAnd,
                                lhs: Box::new(lhs.unwrap()),
                                rhs: Box::new(rhs.unwrap()),
                            },
                        ),
                        Rule::eq => self.new_node(
                            self.location(&op),
                            NodeKind::BinaryOp {
                                op: BinOp::Eq,
                                lhs: Box::new(lhs.unwrap()),
                                rhs: Box::new(rhs.unwrap()),
                            },
                        ),
                        Rule::neq => self.new_node(
                            self.location(&op),
                            NodeKind::BinaryOp {
                                op: BinOp::Neq,
                                lhs: Box::new(lhs.unwrap()),
                                rhs: Box::new(rhs.unwrap()),
                            },
                        ),
                        Rule::gt => self.new_node(
                            self.location(&op),
                            NodeKind::BinaryOp {
                                op: BinOp::Gt,
                                lhs: Box::new(lhs.unwrap()),
                                rhs: Box::new(rhs.unwrap()),
                            },
                        ),
                        Rule::gte => self.new_node(
                            self.location(&op),
                            NodeKind::BinaryOp {
                                op: BinOp::Gte,
                                lhs: Box::new(lhs.unwrap()),
                                rhs: Box::new(rhs.unwrap()),
                            },
                        ),
                        Rule::lt => self.new_node(
                            self.location(&op),
                            NodeKind::BinaryOp {
                                op: BinOp::Lt,
                                lhs: Box::new(lhs.unwrap()),
                                rhs: Box::new(rhs.unwrap()),
                            },
                        ),
                        Rule::lte => self.new_node(
                            self.location(&op),
                            NodeKind::BinaryOp {
                                op: BinOp::Lte,
                                lhs: Box::new(lhs.unwrap()),
                                rhs: Box::new(rhs.unwrap()),
                            },
                        ),
                        Rule::add => self.new_node(
                            self.location(&op),
                            NodeKind::BinaryOp {
                                op: BinOp::Add,
                                lhs: Box::new(lhs.unwrap()),
                                rhs: Box::new(rhs.unwrap()),
                            },
                        ),
                        Rule::sub => self.new_node(
                            self.location(&op),
                            NodeKind::BinaryOp {
                                op: BinOp::Sub,
                                lhs: Box::new(lhs.unwrap()),
                                rhs: Box::new(rhs.unwrap()),
                            },
                        ),
                        Rule::mul => self.new_node(
                            self.location(&op),
                            NodeKind::BinaryOp {
                                op: BinOp::Mul,
                                lhs: Box::new(lhs.unwrap()),
                                rhs: Box::new(rhs.unwrap()),
                            },
                        ),
                        Rule::div => self.new_node(
                            self.location(&op),
                            NodeKind::BinaryOp {
                                op: BinOp::Div,
                                lhs: Box::new(lhs.unwrap()),
                                rhs: Box::new(rhs.unwrap()),
                            },
                        ),
                        Rule::pipe => self.new_node(
                            self.location(&op),
                            NodeKind::Pipe {
                                lhs: Box::new(lhs.unwrap()),
                                rhs: Box::new(rhs.unwrap()),
                            },
                        ),
                        //Rule::pow => (1..rhs + 1).map(|_| lhs).product(),
                        _ => unreachable!(),
                    })
                    .parse(pair.into_inner())
            }
            Rule::term => {
                let mut iter = pair.into_inner();
                let result = self.to_ast(iter.next().unwrap()).unwrap();

                Some(iter.fold(result, |acc, p| {
                    let loc = self.location(&p);
                    let rule = p.as_rule();
                    self.new_node(
                        loc,
                        match rule {
                            Rule::subscript => {
                                let key = self.to_ast(p.into_inner().next().unwrap()).unwrap();
                                NodeKind::Subscript {
                                    object: Box::new(acc),
                                    key: Box::new(key),
                                }
                            }
                            Rule::field_access => {
                                let key = self.to_ast(p.into_inner().next().unwrap()).unwrap();
                                NodeKind::FieldAccess {
                                    object: Box::new(acc),
                                    key: Box::new(key),
                                }
                            }
                            Rule::call => {
                                let args = p
                                    .into_inner()
                                    .next()
                                    .unwrap()
                                    .into_inner()
                                    .filter_map(|p| self.to_ast(p))
                                    .collect();
                                NodeKind::FunctionCall {
                                    function: Box::new(acc),
                                    args,
                                }
                            }
                            _ => unreachable!(),
                        },
                    )
                    .unwrap()
                }))
            }
            Rule::function_literal => {
                let mut iter = pair.into_inner();
                let arg_names: Vec<Node> = iter
                    .next()
                    .unwrap()
                    .into_inner()
                    .filter_map(|p| self.to_ast(p))
                    .collect();

                let body = self.to_ast(iter.next().unwrap())?;

                self.new_node(
                    loc,
                    NodeKind::FunctionLiteral {
                        arg_names,
                        body: Box::new(body),
                    },
                )
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
                    loc,
                    NodeKind::FunctionCall {
                        function: Box::new(func),
                        args,
                    },
                )
            }
            Rule::args => unreachable!(),
            Rule::variable_ref => {
                let identifier = self.to_ast(pair.into_inner().next().unwrap()).unwrap();
                self.new_node(
                    loc,
                    NodeKind::VariableRef {
                        identifier: Box::new(identifier),
                    },
                )
            }
            Rule::tuple_literal => {
                let elements: Vec<Node> =
                    pair.into_inner().filter_map(|p| self.to_ast(p)).collect();
                self.new_node(loc, NodeKind::TupleLiteral(elements))
            }
            Rule::list_literal => {
                let elements: Vec<Node> =
                    pair.into_inner().filter_map(|p| self.to_ast(p)).collect();
                self.new_node(loc, NodeKind::ListLiteral(elements))
            }
            Rule::dict_literal => {
                let elements: Vec<(Node, Node)> = pair
                    .into_inner()
                    .map(|p| {
                        let mut iter = p.into_inner();
                        let key = self.to_ast(iter.next().unwrap()).unwrap();
                        let value = self.to_ast(iter.next().unwrap()).unwrap();
                        (key, value)
                    })
                    .collect();
                self.new_node(loc, NodeKind::DictLiteral(elements))
            }
            Rule::struct_literal => {
                let mut iter = pair.into_inner();
                let module = self
                    .to_ast(iter.next().unwrap().into_inner().next().unwrap())
                    .unwrap();

                let fields: Vec<(Node, Node)> = iter
                    .map(|p| {
                        let mut iter = p.into_inner();
                        let name = self.to_ast(iter.next().unwrap()).unwrap();
                        let value = self.to_ast(iter.next().unwrap()).unwrap();
                        (name, value)
                    })
                    .collect();
                self.new_node(
                    loc,
                    NodeKind::StructLiteral {
                        module: Box::new(module),
                        fields,
                    },
                )
            }
            Rule::symbol_dict_key => {
                let string = pair.into_inner().next().unwrap().as_str().to_owned();
                self.new_node(loc, NodeKind::SymbolLiteral(string.into()))
            }
            Rule::string_literal => {
                let string = unescape(pair.into_inner().next().unwrap().as_str()).unwrap();
                self.new_node(loc, NodeKind::StringLiteral(string))
            }
            Rule::string_contents => panic!(),
            Rule::char => todo!(),
            Rule::number_literal => {
                // TODO: Floats
                self.new_node(
                    loc,
                    NodeKind::IntegerLiteral(pair.as_str().parse::<i64>().unwrap()),
                )
            }
            Rule::bool_literal => self.new_node(
                loc,
                NodeKind::BoolLiteral(pair.as_str().parse::<bool>().unwrap()),
            ),
            Rule::nil_literal => self.new_node(loc, NodeKind::NilLiteral),
            Rule::symbol_literal => {
                let contents = pair.into_inner().next().unwrap();
                match contents.as_rule() {
                    Rule::identifier => {
                        let string = contents.as_str().to_owned();
                        self.new_node(loc, NodeKind::SymbolLiteral(string.into()))
                    }
                    Rule::string_literal => {
                        let string =
                            unescape(contents.into_inner().next().unwrap().as_str()).unwrap();
                        self.new_node(loc, NodeKind::SymbolLiteral(string.into()))
                    }
                    _ => unreachable!(),
                }
            }
            Rule::match_expr => {
                let mut iter = pair.into_inner();
                let expr = self.to_ast(iter.next().unwrap()).unwrap();
                let cases: Vec<Node> = iter
                    .map(|p| {
                        let loc = self.location(&p);
                        let mut iter = p.into_inner();
                        let pattern = self.to_ast(iter.next().unwrap()).unwrap();
                        let body = self.wrap_in_body(self.to_ast(iter.next().unwrap()).unwrap());
                        self.new_node(
                            loc,
                            NodeKind::MatchCase {
                                pattern: Box::new(pattern),
                                body: Box::new(body),
                            },
                        )
                        .unwrap()
                    })
                    .collect();
                self.new_node(
                    loc,
                    NodeKind::Match {
                        expr: Box::new(expr),
                        cases,
                    },
                )
            }
            rule => todo!("implement {:?}", rule),
        }
    }

    fn location(&self, pair: &Pair<Rule>) -> (LineCol, LineCol) {
        (
            pair.line_col().into(),
            pair.as_span().end_pos().line_col().into(),
        )
    }

    fn new_node(&self, location: (LineCol, LineCol), kind: NodeKind) -> Option<Node> {
        let node = Node {
            id: NodeId::default(),
            kind,
            source: Source {
                file: self.source_file_name.clone(),
                start: location.0,
                end: location.1,
            },
        };
        Some(node)
    }

    fn wrap_in_body(&self, node: Node) -> Node {
        if matches!(node.kind, NodeKind::Body(_)) {
            return node;
        }

        self.new_node(
            (node.source.start.clone(), node.source.end.clone()),
            NodeKind::Body(vec![node]),
        )
        .unwrap()
    }
}
