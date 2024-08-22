use crate::{
    exception::{exception, Exception},
    value::Value,
};

pub struct Pattern {
    /// How many stack slots to store the result of the pattern match.
    pub stack_slot_count: usize,
    /// Root node of the pattern.
    pub node: PatternNode,
}

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

    pub fn match_pattern(&self, value: &Value) -> Result<Vec<Value>, Exception> {
        let mut stack = vec![Value::Nil; self.stack_slot_count];

        self.node.match_pattern(value, &mut stack)?;

        Ok(stack)
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
