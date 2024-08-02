#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct Instr {
    pub op: OpCode,
    pub arg0: u8,
    pub arg1: u8,
    pub arg2: u8,
}

#[derive(Debug, Clone, Copy)]
pub enum OpCode {
    /// Constant(index) +1  - pushes constant at arg0 to stack
    Constant,
    /// GetGlobal(module, name) +1 - gets global value and pushes to stack
    GetGlobal,
    /// SetGlobal(module, name) +0 - sets global value to value at top of stack
    SetGlobal,
    /// Call(num_args) -1 -n +1 - calls function
    Call,
    /// Return -1 - returns from function
    Return,
}

impl Instr {
    pub fn constant(index: u8) -> Self {
        Self::new(OpCode::Constant, index, 0, 0)
    }

    pub fn get_global(module: u8, name: u8) -> Self {
        Self::new(OpCode::GetGlobal, module, name, 0)
    }

    pub fn set_global(module: u8, name: u8) -> Self {
        Self::new(OpCode::SetGlobal, module, name, 0)
    }

    pub fn call(num_args: u8) -> Self {
        Self::new(OpCode::Call, num_args, 0, 0)
    }

    pub fn return_() -> Self {
        Self::new(OpCode::Return, 0, 0, 0)
    }

    fn new(op: OpCode, arg0: u8, arg1: u8, arg2: u8) -> Self {
        Instr {
            op,
            arg0,
            arg1,
            arg2,
        }
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn instr_is_4_bytes() {
        assert_eq!(std::mem::size_of::<super::Instr>(), 4);
    }
}
