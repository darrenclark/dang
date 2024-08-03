use crate::value::Value;

use super::{
    chunk::Chunk,
    function::Function,
    inst::{Instr, OpCode},
};

pub fn disassemble(function: &Function) {
    println!("==== {:?} ====", function);
    for instr in function.chunk().code.iter() {
        disassemble_instruction(function.chunk(), instr);
    }
    println!();

    for c in function.chunk().constants.iter() {
        if let Value::Function(f) = c {
            disassemble(f);
        }
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
            op: OpCode::GetLocal,
            arg0,
            ..
        } => println!("GetLocal   \t{}", arg0),
        Instr {
            op: OpCode::SetLocal,
            arg0,
            ..
        } => println!("SetLocal   \t{}", arg0),
        Instr {
            op: OpCode::Call,
            arg0,
            ..
        } => println!("Call \t{}", arg0),
        Instr {
            op: OpCode::Return, ..
        } => println!("Return"),
        Instr {
            op: OpCode::Add, ..
        } => println!("Add"),
        Instr {
            op: OpCode::Sub, ..
        } => println!("Sub"),
        Instr {
            op: OpCode::Mul, ..
        } => println!("Mul"),
        Instr {
            op: OpCode::Div, ..
        } => println!("Div"),
        Instr { op: OpCode::Eq, .. } => println!("Eq"),
        Instr {
            op: OpCode::Neq, ..
        } => println!("Neq"),
        Instr { op: OpCode::Gt, .. } => println!("Gt"),
        Instr {
            op: OpCode::Gte, ..
        } => println!("Gte"),
        Instr { op: OpCode::Lt, .. } => println!("Lt"),
        Instr {
            op: OpCode::Lte, ..
        } => println!("Lte"),
        Instr {
            op: OpCode::Neg, ..
        } => println!("Neg"),
        Instr {
            op: OpCode::LogicalNeg,
            ..
        } => println!("LogicalNeg"),
        Instr {
            op: OpCode::BranchIfTrue,
            ..
        } => println!("BranchIfTrue  {}", instr.wide_arg()),
        Instr {
            op: OpCode::BranchIfFalse,
            ..
        } => println!("BranchIfFalse {}", instr.wide_arg()),
        Instr {
            op: OpCode::Pop, ..
        } => println!("Pop"),
        Instr {
            op: OpCode::Jump, ..
        } => println!("Jump {}", instr.wide_arg()),
        Instr {
            op: OpCode::MakeList,
            ..
        } => println!("MakeList {}", instr.wide_arg()),
        Instr {
            op: OpCode::MakeTuple,
            ..
        } => println!("MakeTuple {}", instr.wide_arg()),
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
