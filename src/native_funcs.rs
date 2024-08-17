use std::rc::Rc;

use pretty::termcolor::{ColorChoice, StandardStream};
use regex::Regex;
use ustr::ustr;

use crate::exception::{exception, Exception};
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
        // lists
        ("append", append),
        ("sort_impl", sort_impl),
        ("reverse", reverse),
        // strings
        ("split", split),
        ("trim", trim),
        ("str_replace", str_replace),
        ("str_reverse", str_reverse),
        ("str_find", str_find),
        ("str_rfind", str_rfind),
        ("str_starts_with?", str_starts_with),
        ("str_ends_with?", str_ends_with),
        // regex
        ("regex_match", regex_match),
        ("regex_find", regex_find),
        ("regex_find_index", regex_find_index),
        // casts
        ("int", int),
        ("str", str_),
        ("symbol", symbol),
        // type checking
        ("type", type_),
        ("is_nil", is_nil),
        ("is_symbol", is_symbol),
        ("is_bool", is_bool),
        ("is_string", is_string),
        ("is_int", is_int),
        ("is_function", is_function),
        ("is_tuple", is_tuple),
        ("is_list", is_list),
        ("is_dict", is_dict),
        ("is_struct", is_struct),
        // misc
        ("raise", raise),
        // functions implemented in Rust for performance reasons
        ("_range_iter", _range_iter),
        // Math
        ("rem", rem),
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

    match std::fs::read_to_string(path.as_str()) {
        Ok(contents) => Ok(Value::string(contents)),
        Err(reason) => {
            exception!("failed to read {}: {}", path, reason)
        }
    }
}
fn inspect(args: &[Value]) -> Result<Value, Exception> {
    for v in args {
        v.to_doc()
            .render_colored(80, StandardStream::stdout(ColorChoice::Auto))
            .unwrap();
        println!()
    }

    Ok(args.first().cloned().unwrap_or(Value::Nil))
}

fn len(args: &[Value]) -> Result<Value, Exception> {
    if args.len() != 1 {
        exception!("len(enumerable) expected one arg")
    }

    if let Value::Tuple(v) = &args[0] {
        return Ok(v.len().into());
    } else if let Value::Dict(v) = &args[0] {
        return Ok(v.len().into());
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

fn append(args: &[Value]) -> Result<Value, Exception> {
    if args.len() != 2 {
        exception!("append(list, item) expected two args")
    }
    let list = match &args[0] {
        Value::List(l) => Rc::clone(l),
        _ => exception!("expected list at arg 0"),
    };
    let mut new_list = list.as_ref().clone();
    new_list.push(args[1].clone());
    Ok(Value::List(Rc::new(new_list)))
}

fn reverse(args: &[Value]) -> Result<Value, Exception> {
    if args.len() != 1 {
        exception!("reverse(list) expected one arg")
    }
    let list = match &args[0] {
        Value::List(l) => Rc::clone(l),
        _ => exception!("expected list at arg 0"),
    };
    let mut new_list = list.as_ref().clone();
    new_list.reverse();
    Ok(Value::List(Rc::new(new_list)))
}

fn sort_impl(args: &[Value]) -> Result<Value, Exception> {
    if args.len() != 1 {
        exception!("sort(list) expected one arg")
    }
    let list = match &args[0] {
        Value::List(l) => Rc::clone(l),
        _ => exception!("expected list at arg 0"),
    };
    let mut new_list = list.as_ref().clone();
    new_list.sort();
    Ok(Value::List(Rc::new(new_list)))
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

    let res: Vec<Value> = string.split(splitter.as_str()).map(Value::string).collect();

    Ok(Value::List(Rc::new(res)))
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
        Value::String(s) => s.as_str(),
        _ => exception!("expected string at arg 0"),
    };
    let pattern = match &args[1] {
        Value::String(s) => s.as_str(),
        _ => exception!("expected string at arg 1"),
    };
    let replacement = match &args[2] {
        Value::String(s) => s.as_str(),
        _ => exception!("expected string at arg 2"),
    };

    let res = string.replace(pattern, replacement);
    Ok(Value::string(res))
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
        Value::String(s) => s.as_str(),
        _ => exception!("expected string at arg 0"),
    };
    let pattern = match &args[1] {
        Value::String(s) => s.as_str(),
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
        Value::String(s) => s.as_str(),
        _ => exception!("expected string at arg 0"),
    };
    let pattern = match &args[1] {
        Value::String(s) => s.as_str(),
        _ => exception!("expected string at arg 1"),
    };

    let res = string
        .rfind(pattern)
        .and_then(|i| byte_to_char_index(string, i))
        .map(|i| Value::Integer(i as i64))
        .unwrap_or(Value::Nil);
    Ok(res)
}

fn str_starts_with(args: &[Value]) -> Result<Value, Exception> {
    if args.len() != 2 {
        exception!("str_starts_with?(string, prefix) expected two args")
    }
    let string = match &args[0] {
        Value::String(s) => s.as_str(),
        _ => exception!("expected string at arg 0"),
    };
    let prefix = match &args[1] {
        Value::String(s) => s.as_str(),
        _ => exception!("expected string at arg 1"),
    };
    Ok(string.starts_with(prefix).into())
}

fn str_ends_with(args: &[Value]) -> Result<Value, Exception> {
    if args.len() != 2 {
        exception!("str_ends_with?(string, suffix) expected two args")
    }
    let string = match &args[0] {
        Value::String(s) => s.as_str(),
        _ => exception!("expected string at arg 0"),
    };
    let suffix = match &args[1] {
        Value::String(s) => s.as_str(),
        _ => exception!("expected string at arg 1"),
    };
    Ok(string.ends_with(suffix).into())
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

fn str_(args: &[Value]) -> Result<Value, Exception> {
    if args.len() != 1 {
        exception!("str(other) expected one arg")
    }
    args[0].cast_to_string()
}

fn symbol(args: &[Value]) -> Result<Value, Exception> {
    if args.len() != 1 {
        exception!("symbol(other) expected one arg")
    }
    args[0].cast_to_symbol()
}

fn type_(args: &[Value]) -> Result<Value, Exception> {
    if args.len() != 1 {
        exception!("type(other) expected one arg")
    }
    Ok(args[0].get_type().as_ref().into())
}

fn is_nil(args: &[Value]) -> Result<Value, Exception> {
    if args.len() != 1 {
        exception!("is_nil(other) expected one arg")
    }
    Ok(args[0].is_nil().into())
}

fn is_symbol(args: &[Value]) -> Result<Value, Exception> {
    if args.len() != 1 {
        exception!("is_symbol(other) expected one arg")
    }
    Ok(args[0].is_symbol().into())
}

fn is_bool(args: &[Value]) -> Result<Value, Exception> {
    if args.len() != 1 {
        exception!("is_bool(other) expected one arg")
    }
    Ok(args[0].is_bool().into())
}

fn is_string(args: &[Value]) -> Result<Value, Exception> {
    if args.len() != 1 {
        exception!("is_string(other) expected one arg")
    }
    Ok(args[0].is_string().into())
}

fn is_int(args: &[Value]) -> Result<Value, Exception> {
    if args.len() != 1 {
        exception!("is_int(other) expected one arg")
    }
    Ok(args[0].is_int().into())
}

fn is_function(args: &[Value]) -> Result<Value, Exception> {
    if args.len() != 1 {
        exception!("is_function(other) expected one arg")
    }
    Ok(args[0].is_function().into())
}

fn is_tuple(args: &[Value]) -> Result<Value, Exception> {
    match args {
        // is_tuple(value, arity)
        [Value::Tuple(t), Value::Integer(l)] => Ok(Value::Bool(t.len() == *l as usize)),
        [_, Value::Integer(_)] => Ok(Value::Bool(false)),
        [_, _] => exception!("is_tuple(value, arity) expected arity to be an integer"),
        // is_tuple(value)
        [value] => Ok(value.is_tuple().into()),
        _ => exception!("is_tuple(value [, arity]) expected one or two args"),
    }
}

fn is_list(args: &[Value]) -> Result<Value, Exception> {
    if args.len() != 1 {
        exception!("is_list(other) expected one arg")
    }
    Ok(args[0].is_list().into())
}

fn is_dict(args: &[Value]) -> Result<Value, Exception> {
    if args.len() != 1 {
        exception!("is_dict(other) expected one arg")
    }
    Ok(args[0].is_dict().into())
}

fn is_struct(args: &[Value]) -> Result<Value, Exception> {
    if args.len() != 1 {
        exception!("is_struct(other) expected one arg")
    }
    Ok(args[0].is_struct().into())
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

fn _range_iter(args: &[Value]) -> Result<Value, Exception> {
    let (start, stop, step) = match args {
        [Value::Integer(start), Value::Integer(stop), Value::Integer(step)] => {
            (*start, *stop, *step)
        }
        _ => exception!("_range_iter(start, stop, step) expected 3 integers"),
    };

    let mut i = start;

    let iter = Value::closure(move |args| {
        if !args.is_empty() {
            exception!("expected zero args to iterator function")
        }

        if i < stop {
            let res = i;
            i += step;
            Ok(Value::Tuple(vec![Value::Symbol(ustr("some")), res.into()]))
        } else {
            Ok(Value::Nil)
        }
    });

    Ok(iter)
}

fn rem(args: &[Value]) -> Result<Value, Exception> {
    if args.len() != 2 {
        exception!("rem(a, b) expected two args")
    }
    let a = match &args[0] {
        Value::Integer(i) => *i,
        _ => exception!("expected int at arg 0"),
    };
    let b = match &args[1] {
        Value::Integer(i) => *i,
        _ => exception!("expected int at arg 1"),
    };

    if b == 0 {
        exception!("rem: division by zero")
    }

    Ok((a % b).into())
}
