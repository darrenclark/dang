use crate::{
    ast::{Node, NodeKind},
    exception::{exception, Exception},
    value::Value,
};

#[derive(Debug, Clone)]
pub struct Pattern {
    /// How many stack slots to store the result of the pattern match.
    pub stack_slot_count: usize,
    /// Root node of the pattern.
    pub node: PatternNode,
}

#[derive(Debug, Clone)]
pub enum PatternNode {
    /// A variable, eg. `x`
    Variable { stack_slot: usize },
    /// A wildcard, eg. `_`
    Wildcard,
    /// A constant, eg. `5`, `true`, `"hello"`, etc.
    Constant(Value),
    /// A tuple, eg. `(x, y)`
    Tuple(Vec<PatternNode>),
    /// A list, with matching on individual elements, eg. `[x, y, z.., z]`
    List {
        start: Vec<PatternNode>,
        rest: Option<Box<PatternNode>>,
        end: Vec<PatternNode>,
    },
}

impl Pattern {
    pub fn new(stack_slot_count: usize, node: PatternNode) -> Self {
        Self {
            stack_slot_count,
            node,
        }
    }

    pub fn from_ast(node: &Node) -> Self {
        let mut top_stack_slot = 0;
        let node = PatternNode::from_ast(node, &mut top_stack_slot);
        Pattern::new(top_stack_slot, node)
    }

    pub fn is_complex_pattern(node: &Node) -> bool {
        !matches!(node.kind, NodeKind::PatternIdentifier { .. })
    }

    /// Iteration order is guaranteed to be the same as the order of the variables in the pattern.
    ///
    /// This function must match PatternNode::from_ast(...) ordering
    pub fn variable_nodes_iter(node: &Node) -> impl Iterator<Item = &Node> {
        node.iter()
            .filter(|child| matches!(child.kind, NodeKind::PatternIdentifier { .. }))
    }

    pub fn match_pattern(&self, value: &Value) -> Result<Vec<Value>, Exception> {
        let mut stack = vec![Value::Nil; self.stack_slot_count];

        if let Ok(()) = self.node.match_pattern(value, &mut stack) {
            Ok(stack)
        } else {
            exception!("Pattern match failed, got {}", value)
        }
    }
}

impl PatternNode {
    pub fn match_pattern(&self, value: &Value, stack: &mut Vec<Value>) -> Result<(), Exception> {
        match self {
            PatternNode::Variable { stack_slot } => {
                stack[*stack_slot] = value.clone();
                Ok(())
            }
            PatternNode::Wildcard => Ok(()),
            PatternNode::Constant(c) => {
                if c == value {
                    Ok(())
                } else {
                    exception!("Pattern match failed");
                }
            }
            PatternNode::Tuple(patterns) => {
                if let Value::Tuple(values) = value {
                    if patterns.len() != values.len() {
                        exception!("Pattern match failed");
                    }
                    for (pattern, value) in patterns.iter().zip(values.iter()) {
                        pattern.match_pattern(value, stack)?;
                    }
                    Ok(())
                } else {
                    exception!("Pattern match failed")
                }
            }
            PatternNode::List { start, rest, end } if rest.is_none() => {
                if let Value::List(values) = value {
                    if start.len() + end.len() != values.len() {
                        exception!("Pattern match failed")
                    }
                    let patterns_iter = start.iter().chain(end.iter());
                    for (pattern, value) in patterns_iter.zip(values.iter()) {
                        pattern.match_pattern(value, stack)?;
                    }
                    Ok(())
                } else {
                    exception!("Pattern match failed")
                }
            }
            PatternNode::List { start, rest, end } => {
                if let Value::List(values) = value {
                    if values.len() < start.len() + end.len() {
                        exception!("Pattern match failed")
                    }
                    // match start
                    for (pattern, value) in start.iter().zip(values.iter()) {
                        pattern.match_pattern(value, stack)?;
                    }
                    // match the "rest" portion
                    let rest_values = values[start.len()..values.len() - end.len()].to_vec();
                    let rest_list = Value::list(rest_values);
                    rest.as_deref().unwrap().match_pattern(&rest_list, stack)?;

                    // match end
                    for (pattern, value) in end.iter().rev().zip(values.iter().rev()) {
                        pattern.match_pattern(value, stack)?;
                    }
                    Ok(())
                } else {
                    exception!("Pattern match failed")
                }
            }
        }
    }

    /// This function must match Pattern::variable_nodes_iter(...) ordering
    pub fn from_ast(node: &Node, top_stack_slot: &mut usize) -> Self {
        match &node.kind {
            NodeKind::PatternConstant { value } => {
                PatternNode::Constant(value.compile_time_value().unwrap())
            }
            NodeKind::PatternIdentifier { identifier: _ } => {
                let stack_slot = *top_stack_slot;
                *top_stack_slot += 1;
                PatternNode::Variable { stack_slot }
            }
            NodeKind::PatternTuple { elements } => {
                let patterns = elements
                    .iter()
                    .map(|element| PatternNode::from_ast(element, top_stack_slot))
                    .collect();
                PatternNode::Tuple(patterns)
            }
            NodeKind::PatternList { items } => {
                let start = items
                    .iter()
                    .map(|item| PatternNode::from_ast(item, top_stack_slot))
                    .collect();
                PatternNode::List {
                    start,
                    rest: None,
                    end: vec![],
                }
            }
            NodeKind::PatternWildcard => PatternNode::Wildcard,
            _ => unreachable!("only expected pattern nodes, got: {:?}", node.kind),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_match_variable() {
        let pattern = Pattern::new(1, PatternNode::Variable { stack_slot: 0 });
        let value = Value::Integer(42);
        let result = pattern.match_pattern(&value).unwrap();
        assert_eq!(result, vec![value]);
    }

    #[test]
    fn test_match_wildcard() {
        let pattern = Pattern::new(0, PatternNode::Wildcard);
        let value = Value::Integer(42);
        let result = pattern.match_pattern(&value).unwrap();
        assert_eq!(result, vec![]);
    }

    #[test]
    fn test_match_constant() {
        let pattern = Pattern::new(0, PatternNode::Constant(Value::Integer(42)));
        let value = Value::Integer(42);
        let result = pattern.match_pattern(&value).unwrap();
        assert_eq!(result, vec![]);
    }

    #[test]
    fn test_tuple() {
        let pattern = Pattern::new(
            2,
            PatternNode::Tuple(vec![
                PatternNode::Variable { stack_slot: 0 },
                PatternNode::Variable { stack_slot: 1 },
            ]),
        );
        let value = Value::Tuple(vec![Value::Integer(42), Value::Integer(43)]);
        let result = pattern.match_pattern(&value).unwrap();
        assert_eq!(result, vec![Value::Integer(42), Value::Integer(43)]);
    }

    #[test]
    fn test_list_with_rest_at_end() {
        let pattern = Pattern::new(
            3,
            PatternNode::List {
                start: vec![
                    PatternNode::Variable { stack_slot: 0 },
                    PatternNode::Variable { stack_slot: 1 },
                ],
                rest: Some(Box::new(PatternNode::Variable { stack_slot: 2 })),
                end: vec![],
            },
        );
        let value = Value::list(vec![
            Value::Integer(42),
            Value::Integer(43),
            Value::Integer(44),
            Value::Integer(45),
        ]);
        let result = pattern.match_pattern(&value).unwrap();
        assert_eq!(
            result,
            vec![
                Value::Integer(42),
                Value::Integer(43),
                Value::list(vec![Value::Integer(44), Value::Integer(45)])
            ]
        );
    }

    #[test]
    fn test_list_with_rest_at_start() {
        let pattern = Pattern::new(
            3,
            PatternNode::List {
                start: vec![],
                rest: Some(Box::new(PatternNode::Variable { stack_slot: 0 })),
                end: vec![
                    PatternNode::Variable { stack_slot: 1 },
                    PatternNode::Variable { stack_slot: 2 },
                ],
            },
        );

        let value = Value::list(vec![
            Value::Integer(42),
            Value::Integer(43),
            Value::Integer(44),
            Value::Integer(45),
        ]);

        let result = pattern.match_pattern(&value).unwrap();
        assert_eq!(
            result,
            vec![
                Value::list(vec![Value::Integer(42), Value::Integer(43)]),
                Value::Integer(44),
                Value::Integer(45),
            ]
        );
    }

    #[test]
    fn test_list_with_rest_in_middle() {
        let pattern = Pattern::new(
            3,
            PatternNode::List {
                start: vec![PatternNode::Variable { stack_slot: 0 }],
                rest: Some(Box::new(PatternNode::Variable { stack_slot: 1 })),
                end: vec![PatternNode::Variable { stack_slot: 2 }],
            },
        );
        let value = Value::list(vec![
            Value::Integer(42),
            Value::Integer(43),
            Value::Integer(44),
            Value::Integer(45),
        ]);
        let result = pattern.match_pattern(&value).unwrap();
        assert_eq!(
            result,
            vec![
                Value::Integer(42),
                Value::list(vec![Value::Integer(43), Value::Integer(44)]),
                Value::Integer(45),
            ]
        );
    }
}
