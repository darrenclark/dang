use std::collections::HashSet;

use ustr::Ustr;

use crate::{
    ast::NodeKind,
    exception::{exception, Exception},
    program::Program,
    struct_info::StructInfo,
};

use super::{CompilationState, Compiler};

pub fn struct_info_phase(
    _compiler: &Compiler,
    compilation_state: &mut CompilationState,
    _program: &mut Program,
) -> Result<(), Exception> {
    let struct_def = compilation_state
        .ast()
        .iter()
        .find(|n| matches!(n.kind, NodeKind::StructDef { .. }));

    if struct_def.is_none() {
        return Ok(());
    }

    if !compilation_state
        .ast()
        .iter()
        .any(|n| matches!(n.kind, NodeKind::Module { .. }))
    {
        exception!("structs must be defined in a module")
    }

    let fields = match &struct_def.unwrap().kind {
        NodeKind::StructDef { fields } => fields,
        _ => unreachable!(),
    };

    let mut encountered_fields: HashSet<Ustr> = HashSet::new();
    let mut struct_info = StructInfo::new(compilation_state.module_name);

    for (name, default_value) in fields {
        let name = Ustr::from(name.unwrap_identifier());

        if encountered_fields.contains(&name) {
            exception!("duplicate field `{}` in struct", name)
        }
        encountered_fields.insert(name);

        if let Some(n) = default_value {
            let v = n.compile_time_value();
            if v.is_none() {
                exception!("field `{}` in struct not a compile time constant", name)
            }
            struct_info.fields.push(name);
            struct_info.default_values.insert(name, v.unwrap());
        } else {
            struct_info.fields.push(name);
            struct_info.required_fields.insert(name);
        }
    }

    compilation_state.struct_info = Some(struct_info);

    Ok(())
}
