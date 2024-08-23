use crate::value::Value;

use super::{
    chunk::Chunk,
    function::Function,
    inst::{Instr, OpCode},
};

pub fn disassemble(function: &Function) {
    println!("==== {} ====", function.get_debug_name());
    for i in 0..function.chunk().code.len() {
        disassemble_instruction(function.chunk(), i);
    }
    println!();

    for c in function.chunk().constants.iter() {
        if let Value::Function(f) = c {
            if f.chunk().source_file == function.chunk().source_file {
                disassemble(f);
            }
        }
    }
}

pub fn disassemble_instruction(chunk: &Chunk, i: usize) {
    let instr = &chunk.code[i];

    let line = &chunk.source_locs[i].line;
    print!("[line {:>2}] ", line);

    match instr {
        Instr {
            op: OpCode::VmArg,
            arg0,
            ..
        } => println!("VmArg   {}", arg0),
        Instr {
            op: OpCode::Constant,
            arg0,
            ..
        } => println!("Constant   {} \t\t{}", arg0, consts(chunk, &[*arg0])),
        Instr {
            op: OpCode::PushNil,
            ..
        } => println!("PushNil"),
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
            op: OpCode::GlobalIsDefined,
            arg0,
            arg1,
            ..
        } => println!(
            "GlobalIsDefined  {}, {} \t{}",
            arg0,
            arg1,
            consts(chunk, &[*arg0, *arg1])
        ),
        Instr {
            op: OpCode::GetUpvalue,
            arg0,
            ..
        } => println!("GetUpvalue   \t{}", arg0),
        Instr {
            op: OpCode::SetUpvalue,
            arg0,
            ..
        } => println!("SetUpvalue   \t{}", arg0),
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
            op: OpCode::TailCall,
            arg0,
            ..
        } => println!("TailCall \t{}", arg0),
        Instr {
            op: OpCode::Return, ..
        } => println!("Return"),
        Instr {
            op: OpCode::Closure,
            ..
        } => println!("Closure"),
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
            op: OpCode::PopLocals,
            ..
        } => println!("PopLocals {}", instr.wide_arg()),
        Instr {
            op: OpCode::Jump, ..
        } => println!("Jump {}", instr.wide_arg()),
        Instr {
            op: OpCode::JumpBack,
            ..
        } => println!("JumpBack -{}", instr.wide_arg()),
        Instr {
            op: OpCode::MakeList,
            ..
        } => println!("MakeList {}", instr.wide_arg()),
        Instr {
            op: OpCode::MakeTuple,
            ..
        } => println!("MakeTuple {}", instr.wide_arg()),
        Instr {
            op: OpCode::MakeDict,
            ..
        } => println!("MakeDict {}", instr.wide_arg()),
        Instr {
            op: OpCode::MakeStruct,
            arg0,
            arg1,
            ..
        } => println!("MakeStruct {} {} \t{}", arg0, arg1, consts(chunk, &[*arg0])),
        Instr {
            op: OpCode::GetField,
            ..
        } => println!("GetField"),
        Instr {
            op: OpCode::GetSubscript,
            ..
        } => println!("GetSubscript"),
        Instr {
            op: OpCode::SetField,
            ..
        } => println!("SetField"),
        Instr {
            op: OpCode::SetSubscript,
            ..
        } => println!("SetSubscript"),
        Instr {
            op: OpCode::Dup,
            arg0,
            ..
        } => println!("Dup {}", arg0),
        Instr {
            op: OpCode::GetIter,
            ..
        } => println!("GetIter"),
        Instr {
            op: OpCode::CallIter,
            ..
        } => println!("CallIter"),
        Instr {
            op: OpCode::ForIter,
            ..
        } => println!("ForIter {}", instr.wide_arg()),
        Instr {
            op: OpCode::UnpackTuple,
            arg0,
            ..
        } => println!("UnpackTuple {}", arg0),
        Instr {
            op: OpCode::Match,
            arg0,
            ..
        } => println!("Match {}     {:?}", arg0, pattern(chunk, *arg0)),
    }
}

fn consts(chunk: &Chunk, constants: &[u8]) -> String {
    constants
        .iter()
        .map(|c| match chunk.constants.get(*c as usize) {
            None => "{INVALID_CONSTANT}".to_owned(),
            Some(v @ Value::Symbol(_)) => format!("{}", v),
            Some(Value::Function(f)) => f.get_debug_name().to_owned(),
            Some(v) => format!("{:?}", v),
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn pattern(chunk: &Chunk, pattern_index: u8) -> String {
    if let Some(pattern) = chunk.patterns.get(pattern_index as usize) {
        return format!("{:?}", pattern);
    } else {
        "{INVALID_PATTERN}".to_owned()
    }
}
