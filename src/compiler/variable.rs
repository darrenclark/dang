use ustr::Ustr;

use crate::module::ModuleName;

/// Represents a variable definition
#[derive(Debug, Clone, Default)]
pub struct Variable {
    pub closed_over: bool,
}

/// Represents a reference to a variable
#[derive(Debug, Clone)]
pub struct VariableRef {
    pub allocation: VariableAllocation,
}

/// Represents the technical details on how to access a variable
/// Needs a new name
#[derive(Debug, Clone)]
pub enum VariableAllocation {
    Global {
        module: ModuleName,
        name: Ustr,
    },
    Upvalue {
        source: UpvalueSource,
        upvalue_index: usize,
    },
    /// index relative to the frame base
    Local {
        index: usize,
    },
}

#[derive(Debug, Clone)]
pub enum UpvalueSource {
    /// Upvalue is a local captured from the parent function.  Index is relative to base of the
    /// caller's stack frame.
    Local { stack_index_relative_to_base: usize },
    /// Upvalue is an upvalue already captured by the parent function.  Index is the upvalue index
    /// of
    Upvalue { upvalue_index: usize },
}
