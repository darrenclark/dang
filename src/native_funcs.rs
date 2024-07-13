use regex::Regex;

use crate::interpreter::{exception, Exception};
use crate::string_utils::byte_to_char_index;
use crate::value::Value;

type NativeFuncPtr = fn(&[Value]) -> Result<Value, Exception>;

pub fn funcs() -> Vec<(&'static str, NativeFuncPtr)> {
    vec![
        // IO
        ("print", print),
        ("println", println),
        ("inspect", inspect),
        ("readfile", readfile),
        // enumerables
        ("len", len),
        ("get", get),
        // strings
        ("split", split),
        ("trim", trim),
        ("str_replace", str_replace),
        ("str_reverse", str_reverse),
        ("str_find", str_find),
        ("str_rfind", str_rfind),
        // regex
        ("regex_match", regex_match),
        ("regex_find", regex_find),
        ("regex_find_index", regex_find_index),
        // casts
        ("int", int),
        // misc
        ("raise", raise),
    ]
}

pub fn func(name: &str) -> Option<NativeFuncPtr> {
    funcs()
        .iter()
        .find_map(|(n, ptr)| if *n == name { Some(ptr) } else { None })
        .copied()
}

fn print(args: &[Value]) -> Result<Value, Exception> {
    print_impl(args, false)
}

fn println(args: &[Value]) -> Result<Value, Exception> {
    print_impl(args, true)
}

fn print_impl(args: &[Value], newline: bool) -> Result<Value, Exception> {
    for v in args {
        if let Value::String(s) = v {
            print!("{}", s)
        } else if let Value::Integer(i) = v {
            print!("{}", i)
        } else {
            exception!("print: arg is not printable: {:?}", v)
        }
    }

    if newline {
        println!();
    }

    Ok(Value::Nil)
}

fn readfile(args: &[Value]) -> Result<Value, Exception> {
    if args.len() != 1 {
        exception!("readfile(path) expected one arg")
    }
    let path = match &args[0] {
        Value::String(p) => p,
        _ => exception!("expected path as a string"),
    };

    match std::fs::read_to_string(path) {
        Ok(contents) => Ok(Value::String(contents)),
        Err(reason) => {
            exception!("failed to read {}: {}", path, reason)
        }
    }
}

fn inspect(args: &[Value]) -> Result<Value, Exception> {
    for v in args {
        println!("{:?}", v)
    }

    Ok(args.first().cloned().unwrap_or(Value::Nil))
}

fn len(args: &[Value]) -> Result<Value, Exception> {
    if args.len() != 1 {
        exception!("len(enumerable) expected one arg")
    }
    args[0].ensure_enumerable("len")?;

    Ok(Value::Integer(args[0].enum_len() as i64))
}

fn get(args: &[Value]) -> Result<Value, Exception> {
    if args.len() < 2 {
        exception!("get(enumerable, path..) expected at least two args")
    }

    let mut res = args[0].clone();
    for index in args.iter().skip(1) {
        res = res.at(index)?;
    }
    Ok(res)
}

fn split(args: &[Value]) -> Result<Value, Exception> {
    if args.len() != 2 {
        exception!("split(string, splitter) expected two args")
    }
    let string = match &args[0] {
        Value::String(s) => s,
        _ => exception!("expected string at arg 0"),
    };
    let splitter = match &args[1] {
        Value::String(s) => s,
        _ => exception!("expected string at arg 1"),
    };

    let res: Vec<Value> = string
        .split(splitter)
        .map(|s| Value::String(s.to_owned()))
        .collect();

    Ok(Value::List(res))
}

fn trim(args: &[Value]) -> Result<Value, Exception> {
    let string = match args {
        [Value::String(string)] => string,
        _ => exception!("trim(string) expected 1 string"),
    };

    Ok(string.trim().into())
}

fn str_replace(args: &[Value]) -> Result<Value, Exception> {
    if args.len() != 3 {
        exception!("str_replace(string, pattern, replacement) expected three args")
    }
    let string = match &args[0] {
        Value::String(s) => s,
        _ => exception!("expected string at arg 0"),
    };
    let pattern = match &args[1] {
        Value::String(s) => s,
        _ => exception!("expected string at arg 1"),
    };
    let replacement = match &args[2] {
        Value::String(s) => s,
        _ => exception!("expected string at arg 2"),
    };

    let res = string.replace(pattern, replacement);
    Ok(Value::String(res))
}

fn str_reverse(args: &[Value]) -> Result<Value, Exception> {
    let string = match args {
        [Value::String(string)] => string,
        _ => exception!("str_reverse(string) expected 1 string"),
    };

    let reversed: String = string.chars().rev().collect();
    Ok(reversed.into())
}

fn str_find(args: &[Value]) -> Result<Value, Exception> {
    if args.len() != 2 {
        exception!("str_find(string, pattern) expected two args")
    }
    let string = match &args[0] {
        Value::String(s) => s,
        _ => exception!("expected string at arg 0"),
    };
    let pattern = match &args[1] {
        Value::String(s) => s,
        _ => exception!("expected string at arg 1"),
    };

    let res = string
        .find(pattern)
        .and_then(|i| byte_to_char_index(string, i))
        .map(|i| Value::Integer(i as i64))
        .unwrap_or(Value::Nil);
    Ok(res)
}

fn str_rfind(args: &[Value]) -> Result<Value, Exception> {
    if args.len() != 2 {
        exception!("str_find(string, pattern) expected two args")
    }
    let string = match &args[0] {
        Value::String(s) => s,
        _ => exception!("expected string at arg 0"),
    };
    let pattern = match &args[1] {
        Value::String(s) => s,
        _ => exception!("expected string at arg 1"),
    };

    let res = string
        .rfind(pattern)
        .and_then(|i| byte_to_char_index(string, i))
        .map(|i| Value::Integer(i as i64))
        .unwrap_or(Value::Nil);
    Ok(res)
}

fn regex_match(args: &[Value]) -> Result<Value, Exception> {
    let (string, pattern) = match args {
        [Value::String(string), Value::String(pattern)] => (string, pattern),
        _ => exception!("regex_match(string, pattern) expected 2 strings"),
    };

    let regex = Regex::new(pattern).map_err(|e| Exception::new(format!("{}", e)))?;
    Ok(regex.is_match(string).into())
}

fn regex_find(args: &[Value]) -> Result<Value, Exception> {
    let (string, pattern) = match args {
        [Value::String(string), Value::String(pattern)] => (string, pattern),
        _ => exception!("regex_find(string, pattern) expected 2 strings"),
    };

    let regex = Regex::new(pattern).map_err(|e| Exception::new(format!("{}", e)))?;
    Ok(regex.find(string).map(|m| m.as_str()).into())
}

fn regex_find_index(args: &[Value]) -> Result<Value, Exception> {
    let (string, pattern) = match args {
        [Value::String(string), Value::String(pattern)] => (string, pattern),
        _ => exception!("regex_find_index(string, pattern) expected 2 strings"),
    };

    let regex = Regex::new(pattern).map_err(|e| Exception::new(format!("{}", e)))?;
    Ok(regex.find(string).map(|m| vec![m.start(), m.end()]).into())
}

fn int(args: &[Value]) -> Result<Value, Exception> {
    if args.len() != 1 {
        exception!("int(other) expected one arg")
    }
    args[0].cast_to_int()
}

fn raise(args: &[Value]) -> Result<Value, Exception> {
    if args.len() != 1 {
        exception!("expected one string arg to raise")
    }
    if let Value::String(s) = &args[0] {
        exception!("{}", s)
    }
    exception!("expected one string arg to raise")
}
