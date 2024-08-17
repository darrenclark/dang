use std::collections::HashSet;

use ustr::Ustr;

use crate::{
    ast::{AstWalker, Node, NodeKind},
    exception::{exception, Exception},
    program::Program,
    struct_info::StructInfo,
};

use super::{resolve_structs_phase::ReferencedStruct, CompilationState, Compiler};

pub fn validate_struct_fields_phase(
    _compiler: &Compiler,
    compilation_state: &mut CompilationState,
    _program: &mut Program,
) -> Result<(), Exception> {
    let ast = compilation_state.ast().clone();

    let mut phase = ValidateStructFieldsPhase {
        compilation_state,
        errored: false,
    };

    ast.walk(&mut phase);

    if phase.errored {
        exception!("errors validating struct fields")
    }

    Ok(())
}

struct ValidateStructFieldsPhase<'a> {
    compilation_state: &'a mut CompilationState,
    errored: bool,
}

impl ValidateStructFieldsPhase<'_> {
    fn validate(&self, struct_info: &StructInfo, fields: &HashSet<&str>) -> Result<(), Exception> {
        for f in struct_info.required_fields.iter() {
            if !fields.contains(&f.as_str()) {
                exception!("required struct field `{}` not provided", f)
            }
        }

        for f in fields.iter() {
            if !struct_info.fields.contains(&Ustr::from(f)) {
                exception!(
                    "unknown field `{}` provided for struct `{}`",
                    f,
                    struct_info.module_name
                )
            }
        }

        Ok(())
    }
}

impl<'a> AstWalker for ValidateStructFieldsPhase<'a> {
    fn enter_node(&mut self, node: &Node) {
        if let NodeKind::StructLiteral { module: _, fields } = &node.kind {
            let referenced_struct = self
                .compilation_state
                .tags
                .get::<ReferencedStruct>(node.id)
                .unwrap();

            let fields = fields.iter().map(|(n, _)| n.unwrap_identifier()).collect();

            match self.validate(&referenced_struct.struct_info, &fields) {
                Ok(_) => {}
                Err(mut e) => {
                    e.set_source_from_node(node);
                    self.compilation_state.errors.push(e);
                    self.errored = true;
                }
            }
        }
    }
}
