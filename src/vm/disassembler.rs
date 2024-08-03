use super::{
    chunk::Chunk,
    inst::{Instr, OpCode},
};

pub fn disassemble(chunk: &Chunk) {
    for instr in chunk.code.iter() {
        disassemble_instruction(chunk, instr);
    }
}

fn disassemble_instruction(chunk: &Chunk, instr: &Instr) {
    match instr {
        Instr {
            op: OpCode::Constant,
            arg0,
            ..
        } => println!("Constant   {} \t\t{}", arg0, consts(chunk, &[*arg0])),
        Instr {
            op: OpCode::GetGlobal,
            arg0,
            arg1,
            ..
        } => println!(
            "GetGlobal  {}, {} \t{}",
            arg0,
            arg1,
            consts(chunk, &[*arg0, *arg1])
        ),
        Instr {
            op: OpCode::SetGlobal,
            arg0,
            arg1,
            ..
        } => println!(
            "SetGlobal  {}, {} \t{}",
            arg0,
            arg1,
            consts(chunk, &[*arg0, *arg1])
        ),
        Instr {
            op: OpCode::Call,
            arg0,
            ..
        } => println!("Call \t{}", arg0),
        Instr {
            op: OpCode::Return, ..
        } => println!("Return"),
    }
}

fn consts(chunk: &Chunk, constants: &[u8]) -> String {
    constants
        .iter()
        .map(|c| match chunk.constants.get(*c as usize) {
            None => "{INVALID_CONSTANT}".to_owned(),
            Some(v) => format!("{:?}", v),
        })
        .collect::<Vec<_>>()
        .join(", ")
}
