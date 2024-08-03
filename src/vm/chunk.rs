use crate::value::Value;

use super::inst::Instr;

#[derive(Debug, Clone, Default)]
pub struct Chunk {
    pub code: Vec<Instr>,
    pub constants: Vec<Value>,
}

impl Chunk {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn write(&mut self, instr: Instr) -> usize {
        self.code.push(instr);
        self.code.len() - 1
    }

    pub fn write_constant(&mut self, value: Value) -> u8 {
        self.constants.push(value);
        (self.constants.len() - 1) as u8
    }

    pub fn patch_jump(&mut self, jump_instr_offset: usize) {
        let offset = self.code.len() - jump_instr_offset - 1;
        let jump_instr = &mut self.code[jump_instr_offset];
        jump_instr.set_wide_arg(offset);
    }
}
