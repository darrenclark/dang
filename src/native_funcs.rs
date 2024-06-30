use crate::interpreter::{exception, Exception, Value};

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
        // casts
        ("int", int),
        // misc
        ("raise", raise),
    ]
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
        res.ensure_enumerable("get")?;
        res = res.enum_at(index.to_index()?)?;
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
