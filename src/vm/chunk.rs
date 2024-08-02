use crate::value::Value;

use super::inst::Instr;

#[derive(Debug, Default)]
pub struct Chunk {
    pub code: Vec<Instr>,
    pub constants: Vec<Value>,
}

impl Chunk {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn write(&mut self, instr: Instr) -> u32 {
        self.code.push(instr);
        (self.code.len() - 1) as u32
    }

    pub fn write_constant(&mut self, value: Value) -> u8 {
        self.constants.push(value);
        (self.constants.len() - 1) as u8
    }
}
