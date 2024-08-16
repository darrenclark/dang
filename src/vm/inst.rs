use strum::EnumCount;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct Instr {
    pub op: OpCode,
    pub arg0: u8,
    pub arg1: u8,
    pub arg2: u8,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, EnumCount, strum::Display, strum::FromRepr)]
pub enum OpCode {
    /// VmArg(arg) -0 +1 - fetches an arg provided to the VM.  `arg` corresponds to a number in
    /// this table:
    ///
    ///   1: argv
    VmArg,
    /// Constant(index) +1  - pushes constant at arg0 to stack
    Constant,
    /// PushNil +1 - pushes a nil to stack
    PushNil,
    /// GetGlobal(module, name) +1 - gets global value and pushes to stack
    GetGlobal,
    /// SetGlobal(module, name) -1 - sets global value to value at top of stack
    SetGlobal,
    /// GlobalIsDefined(module, name) +1 - checks if global is defined (used to guard against
    /// loading a module twice)
    GlobalIsDefined,
    /// GetUpvalue(index) +1 - gets upvalue and pushes to stack
    GetUpvalue,
    /// SetUpvalue(index) -1 - sets upvalue to value at top of stack
    SetUpvalue,
    /// GetLocal(index) +1 - gets local variable and pushes to stack
    GetLocal,
    /// SetLocal(index) -1 - sets local variable to value at top of stack
    SetLocal,
    /// Call(num_args) -1 -n +1 - calls function
    Call,
    /// TailCall(num_args) -1 -n +1 - calls function, replacing current frame
    TailCall,
    /// Return -1 - returns from function
    Return,
    /// Closure -1 +1 - creates a closure from the function at the top of the stack
    Closure,
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
    /// PopLocals(n:24bit) -0 -n - saves top value, then pops n values, then restores top value
    /// Additionally will CloseUpvalue any upvalues in the locals
    PopLocals,
    /// Jump(offset:24bit) -0 +0 - jumps forward to the given offset
    Jump,
    /// JumpBack(offset:24bit) -0 +0 - jumps back to the given offset
    JumpBack,
    /// MakeList(size:24bit) -n +1 - makes a list with n elements
    MakeList,
    /// MakeTuple(size:24bit) -n +1 - makes a tuple with n elements
    MakeTuple,
    /// MakeDict(size:24bit) -n*2 +1 - makes a dictionary with n key value pairs
    MakeDict,
    /// MakeStruct(kind, size) -n*2 +1 - makes a struct with n key value pairs
    /// First argument is index into constants to get struct kind (Symbol)
    /// Second argument is number of key/value pairs on stack
    MakeStruct,
    /// GetField -2 +1 - accesses a field (object.key)
    GetField,
    /// GetSubscript -2 +1 - accesses a field via subscript (object[key])
    GetSubscript,
    /// SetField -3 +1 - sets a field (object.key = value)
    SetField,
    /// SetSubscript -3 +1 - sets a field via subscript (object[key] = value)
    SetSubscript,
    /// Dup(n:24bit) -0 +1 - duplicate element at stack[top - n]
    /// Dup 0 - duplicate top of stack, Dup 1 - duplicate second topmost value in stack, etc.
    Dup,
    /// GetIter -1 +1 - gets iterator for value at top of stack
    GetIter,
    /// CallIter -0, +1 - calls iterator, pushing result - (:some, value) or nil
    CallIter,
    /// ForIter(offset:24bit) -1 +1 - unpacks result of CallIter or jumps to offset when iterator
    /// is exhausted
    ForIter,
    /// UnpackTuple(arity) -1 +arity - unpacks a tuple, popping it & pushing each element
    UnpackTuple,
}

impl Instr {
    pub fn vm_arg(index: u8) -> Self {
        Self::new(OpCode::VmArg, index, 0, 0)
    }

    pub fn constant(index: u8) -> Self {
        Self::new(OpCode::Constant, index, 0, 0)
    }

    pub fn push_nil() -> Self {
        Self::new(OpCode::PushNil, 0, 0, 0)
    }

    pub fn get_global(module: u8, name: u8) -> Self {
        Self::new(OpCode::GetGlobal, module, name, 0)
    }

    pub fn set_global(module: u8, name: u8) -> Self {
        Self::new(OpCode::SetGlobal, module, name, 0)
    }

    pub fn global_is_defined(module: u8, name: u8) -> Self {
        Self::new(OpCode::GlobalIsDefined, module, name, 0)
    }

    pub fn get_upvalue(index: u8) -> Self {
        Self::new(OpCode::GetUpvalue, index, 0, 0)
    }

    pub fn set_upvalue(index: u8) -> Self {
        Self::new(OpCode::SetUpvalue, index, 0, 0)
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

    pub fn tail_call(num_args: u8) -> Self {
        Self::new(OpCode::TailCall, num_args, 0, 0)
    }

    pub fn return_() -> Self {
        Self::new(OpCode::Return, 0, 0, 0)
    }

    pub fn closure() -> Self {
        Self::new(OpCode::Closure, 0, 0, 0)
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
        Self::new_wide(OpCode::BranchIfTrue, offset)
    }

    pub fn branch_if_false(offset: usize) -> Self {
        Self::new_wide(OpCode::BranchIfFalse, offset)
    }

    pub fn pop() -> Self {
        Self::new(OpCode::Pop, 0, 0, 0)
    }

    pub fn pop_locals(n: usize) -> Self {
        Self::new_wide(OpCode::PopLocals, n)
    }

    pub fn jump(offset: usize) -> Self {
        Self::new_wide(OpCode::Jump, offset)
    }

    pub fn jump_back(offset: usize) -> Self {
        Self::new_wide(OpCode::JumpBack, offset)
    }

    pub fn make_list(size: usize) -> Self {
        Self::new_wide(OpCode::MakeList, size)
    }

    pub fn make_tuple(size: usize) -> Self {
        Self::new_wide(OpCode::MakeTuple, size)
    }

    pub fn make_dict(size: usize) -> Self {
        Self::new_wide(OpCode::MakeDict, size)
    }

    pub fn make_struct(kind: u8, size: usize) -> Self {
        Self::new(OpCode::MakeStruct, kind, size as u8, 0)
    }

    pub fn get_field() -> Self {
        Self::new(OpCode::GetField, 0, 0, 0)
    }

    pub fn get_subscript() -> Self {
        Self::new(OpCode::GetSubscript, 0, 0, 0)
    }

    pub fn set_field() -> Self {
        Self::new(OpCode::SetField, 0, 0, 0)
    }

    pub fn set_subscript() -> Self {
        Self::new(OpCode::SetSubscript, 0, 0, 0)
    }

    pub fn dup(n: u8) -> Self {
        Self::new(OpCode::Dup, n, 0, 0)
    }

    pub fn get_iter() -> Self {
        Self::new(OpCode::GetIter, 0, 0, 0)
    }

    pub fn call_iter() -> Self {
        Self::new(OpCode::CallIter, 0, 0, 0)
    }

    pub fn for_iter(end_offset: usize) -> Self {
        Self::new_wide(OpCode::ForIter, end_offset)
    }

    pub fn unpack_tuple(arity: usize) -> Self {
        Self::new(OpCode::UnpackTuple, arity as u8, 0, 0)
    }

    fn new(op: OpCode, arg0: u8, arg1: u8, arg2: u8) -> Self {
        Instr {
            op,
            arg0,
            arg1,
            arg2,
        }
    }

    fn new_wide(op: OpCode, arg: usize) -> Self {
        Instr {
            op,
            arg0: arg as u8,
            arg1: (arg >> 8) as u8,
            arg2: (arg >> 16) as u8,
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
