use std::rc::Rc;

use lazy_static::lazy_static;

use crate::{
    ast::{Node, NodeId, NodeKind, Source},
    interpreter::Exception,
    program::Program,
};

use super::{CompilationState, Compiler};

lazy_static! {}

pub fn insert_prelude_phase(
    compiler: &Compiler,
    compilation_state: &mut CompilationState,
    _program: &mut Program,
) -> Result<(), Exception> {
    if !compilation_state.is_std() {
        let source_file = Rc::new(String::from("<prelude>"));

        match &mut compilation_state.ast_mut().kind {
            NodeKind::SourceFile(children) if children.len() > 0 => {
                let index = if matches!(children[0].kind, NodeKind::Module(_)) {
                    1
                } else {
                    0
                };

                for (i, import) in compiler.implicit_imports.iter().enumerate() {
                    children.insert(
                        index + 1,
                        Node {
                            id: NodeId::default(),
                            kind: NodeKind::Import(import.clone()),
                            source: Source {
                                file: source_file.clone(),
                                line: i,
                                col: 0,
                            },
                        },
                    )
                }
            }
            NodeKind::SourceFile(_) => {}
            _ => unreachable!(),
        }
    }
    Ok(())
}
