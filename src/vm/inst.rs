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
    /// SetGlobal(module, name) -1 - sets global value to value at top of stack
    SetGlobal,
    /// GetLocal(index) +1 - gets local variable and pushes to stack
    GetLocal,
    /// SetLocal(index) -1 - sets local variable to value at top of stack
    SetLocal,
    /// Call(num_args) -1 -n +1 - calls function
    Call,
    /// Return -1 - returns from function
    Return,
    /// Add -2 +1 - adds two values
    Add,
    /// Sub -2 +1 - subtracts two values
    Sub,
    /// Mul -2 +1 - multiplies two values
    Mul,
    /// Div -2 +1 - divides two values
    Div,
    /// Eq -2 +1 - checks equality between two values
    Eq,
    /// Neq -2 +1 - checks inequality between two values
    Neq,
    /// Gt -2 +1 - checks if left value is greater than right value
    Gt,
    /// Gte -2 +1 - checks if left value is greater than or equal to right value
    Gte,
    /// Lt -2 +1 - checks if left value is less than right value
    Lt,
    /// Lte -2 +1 - checks if left value is less than or equal to right value
    Lte,
    /// Neg -1 +1 - negates value
    Neg,
    /// LogicalNeg -1 +1 - negates value
    LogicalNeg,
    /// BranchIfTrue(offset:24bit) -0 +0 - branches if value at top of stack is truthy.
    /// Doesn't pop value.
    BranchIfTrue,
    /// BranchIfFalse(offset:24bit) -0 +0 - branches if value at top of stack is falsey.
    /// Doesn't pop value.
    BranchIfFalse,
    /// Pop -1 +0 - pops value from stack
    Pop,
    /// Jump(offset:24bit) -0 +0 - jumps forward to the given offset
    Jump,
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

    pub fn get_local(index: u8) -> Self {
        Self::new(OpCode::GetLocal, index, 0, 0)
    }

    pub fn set_local(index: u8) -> Self {
        Self::new(OpCode::SetLocal, index, 0, 0)
    }

    pub fn call(num_args: u8) -> Self {
        Self::new(OpCode::Call, num_args, 0, 0)
    }

    pub fn return_() -> Self {
        Self::new(OpCode::Return, 0, 0, 0)
    }

    pub fn add() -> Self {
        Self::new(OpCode::Add, 0, 0, 0)
    }

    pub fn sub() -> Self {
        Self::new(OpCode::Sub, 0, 0, 0)
    }

    pub fn mul() -> Self {
        Self::new(OpCode::Mul, 0, 0, 0)
    }

    pub fn div() -> Self {
        Self::new(OpCode::Div, 0, 0, 0)
    }

    pub fn eq() -> Self {
        Self::new(OpCode::Eq, 0, 0, 0)
    }

    pub fn neq() -> Self {
        Self::new(OpCode::Neq, 0, 0, 0)
    }

    pub fn gt() -> Self {
        Self::new(OpCode::Gt, 0, 0, 0)
    }

    pub fn gte() -> Self {
        Self::new(OpCode::Gte, 0, 0, 0)
    }

    pub fn lt() -> Self {
        Self::new(OpCode::Lt, 0, 0, 0)
    }

    pub fn lte() -> Self {
        Self::new(OpCode::Lte, 0, 0, 0)
    }

    pub fn neg() -> Self {
        Self::new(OpCode::Neg, 0, 0, 0)
    }

    pub fn logical_neg() -> Self {
        Self::new(OpCode::LogicalNeg, 0, 0, 0)
    }

    pub fn branch_if_true(offset: usize) -> Self {
        Self::new(
            OpCode::BranchIfTrue,
            offset as u8,
            (offset >> 8) as u8,
            (offset >> 16) as u8,
        )
    }

    pub fn branch_if_false(offset: usize) -> Self {
        Self::new(
            OpCode::BranchIfFalse,
            offset as u8,
            (offset >> 8) as u8,
            (offset >> 16) as u8,
        )
    }

    pub fn pop() -> Self {
        Self::new(OpCode::Pop, 0, 0, 0)
    }

    pub fn jump(offset: usize) -> Self {
        Self::new(
            OpCode::Jump,
            offset as u8,
            (offset >> 8) as u8,
            (offset >> 16) as u8,
        )
    }

    fn new(op: OpCode, arg0: u8, arg1: u8, arg2: u8) -> Self {
        Instr {
            op,
            arg0,
            arg1,
            arg2,
        }
    }

    pub fn wide_arg(&self) -> usize {
        usize::from(self.arg0) | (usize::from(self.arg1) << 8) | (usize::from(self.arg2) << 16)
    }

    pub fn set_wide_arg(&mut self, offset: usize) {
        self.arg0 = offset as u8;
        self.arg1 = (offset >> 8) as u8;
        self.arg2 = (offset >> 16) as u8;
    }
}

#[cfg(test)]
mod test {
    use super::Instr;

    #[test]
    fn instr_is_4_bytes() {
        assert_eq!(std::mem::size_of::<super::Instr>(), 4);
    }

    #[test]
    fn wide_operands() {
        let instr = Instr::branch_if_true(0x010203);
        assert_eq!(instr.wide_arg(), 0x010203);
    }
}
