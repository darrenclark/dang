use crate::{line_col::LineCol, value::Value};

use super::{inst::Instr, pattern::Pattern};

#[derive(Debug, Clone, Default)]
pub struct Chunk {
    pub code: Vec<Instr>,
    pub constants: Vec<Value>,
    pub patterns: Vec<Pattern>,
    pub source_file: String,
    pub source_locs: Vec<LineCol>,
}

impl Chunk {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn write(&mut self, instr: Instr, source_loc: LineCol) -> usize {
        self.code.push(instr);
        self.source_locs.push(source_loc);
        self.code.len() - 1
    }

    pub fn write_constant(&mut self, value: Value) -> u8 {
        for (i, c) in self.constants.iter().enumerate() {
            if c == &value {
                return i as u8;
            }
        }

        self.constants.push(value);
        (self.constants.len() - 1) as u8
    }

    pub fn write_pattern(&mut self, pattern: Pattern) -> u8 {
        self.patterns.push(pattern);
        (self.patterns.len() - 1) as u8
    }

    pub fn patch_jump(&mut self, jump_instr_offset: usize) {
        let offset = self.code.len() - jump_instr_offset - 1;
        let jump_instr = &mut self.code[jump_instr_offset];
        jump_instr.set_wide_arg(offset);
    }

    pub fn write_jump_back(&mut self, label: usize, source_loc: LineCol) -> usize {
        let offset = self.code.len() - label;
        self.write(Instr::jump_back(offset), source_loc)
    }

    pub fn label(&self, _name: &'static str) -> usize {
        self.code.len()
    }
}
