use crate::{
    ast::{AstWalker, Node, NodeKind},
    interpreter::{exception, Exception},
    program::Program,
    scope::VariableLocation,
    value::Value,
};

pub fn get_pattern_vars(program: &Program, pattern: &Node) -> Vec<VariableLocation> {
    let mut w = GetPatternVars {
        program,
        vars: Vec::new(),
    };
    pattern.walk(&mut w);
    w.vars
}

#[derive(Debug)]
struct GetPatternVars<'a> {
    program: &'a Program,
    vars: Vec<VariableLocation>,
}

impl<'a> AstWalker for GetPatternVars<'a> {
    fn enter_node(&mut self, node: &Node) {
        match &node.kind {
            NodeKind::PatternIdentifier { .. } => {
                let location = self.program.variable_location(node);
                self.vars.push(location)
            }
            NodeKind::PatternTuple { .. } => {}
            _ => unreachable!("only expected pattern nodes, got: {:?}", node),
        }
    }

    fn walk_children(&mut self, node: &Node) -> bool {
        match &node.kind {
            NodeKind::PatternIdentifier { .. } => false,
            NodeKind::PatternTuple { .. } => true,
            _ => unreachable!("only expected pattern nodes, got: {:?}", node),
        }
    }
}

pub fn match_pattern(
    program: &Program,
    pattern: &Node,
    value: Value,
) -> Result<Vec<(VariableLocation, Value)>, Exception> {
    let mut w = MatchPattern {
        program,
        vars: Vec::new(),
    };
    w.do_match(pattern, &value)?;
    Ok(w.vars)
}

#[derive(Debug)]
struct MatchPattern<'a> {
    program: &'a Program,
    vars: Vec<(VariableLocation, Value)>,
}

impl<'a> MatchPattern<'a> {
    fn do_match(&mut self, pattern: &Node, value: &Value) -> Result<(), Exception> {
        match (&pattern.kind, value) {
            (NodeKind::PatternIdentifier { .. }, value) => {
                let location = self.program.variable_location(pattern);
                self.vars.push((location, value.clone()));
            }
            (NodeKind::PatternTuple { elements }, Value::Tuple(values))
                if elements.len() == values.len() =>
            {
                for (element, value) in elements.iter().zip(values.iter()) {
                    let location = self.program.variable_location(element);
                    self.vars.push((location, value.clone()));
                }
            }
            // TODO: improve error message
            (NodeKind::PatternTuple { .. }, _) => exception!("match failure"),
            (_, _) => unreachable!("node not a pattern: {:?}", pattern),
        }
        Ok(())
    }
}
