use std::collections::HashMap;

use pest::iterators::{Pair, Pairs};

use crate::dang_parser::Rule;

#[derive(Debug, Clone)]
pub enum Value {
    Null,
    String(String),
    Integer(i64),
    NativeFunc(&'static str),
}

#[derive(Debug)]
struct Context {
    bindings: HashMap<String, Value>,
}

impl Context {
    fn new() -> Context {
        let mut c = Context {
            bindings: HashMap::new(),
        };

        c.put("add", Value::NativeFunc("add"));
        c.put("print", Value::NativeFunc("print"));
        c.put("println", Value::NativeFunc("println"));

        c
    }

    fn get(&self, binding: &str) -> Option<&Value> {
        self.bindings.get(binding)
    }

    fn put(&mut self, binding: &str, value: Value) {
        self.bindings.insert(String::from(binding), value);
    }
}

pub fn interpret(pairs: Pairs<Rule>) -> Value {
    let mut context = Context::new();
    let mut result = Value::Null;
    for p in pairs {
        if p.as_rule() == Rule::EOI {
            break;
        }
        result = eval(&mut context, p)
    }
    result
}

fn eval(context: &mut Context, pair: Pair<Rule>) -> Value {
    match pair.as_rule() {
        Rule::EOI => panic!(),
        Rule::WHITESPACE => panic!(),
        Rule::source_file => panic!(),
        Rule::identifier => match context.get(pair.as_str()) {
            Some(v) => v.clone(),
            None => panic!("no binding {}", pair.as_str()),
        },
        Rule::stmt => panic!(),
        Rule::expr => panic!(),
        Rule::function_call => {
            let mut iter = pair.into_inner();
            let func = eval(context, iter.next().unwrap());

            if let Value::NativeFunc(name) = func {
                let args = iter
                    .next()
                    .unwrap()
                    .into_inner()
                    .map(|p| eval(context, p))
                    .collect();
                native_call(name, args)
            } else {
                panic!("tried to call a non-function value: {:?}", func)
            }
        }
        Rule::args => panic!(),
        Rule::string_literal => {
            Value::String(String::from(pair.into_inner().next().unwrap().as_str()))
        }
        Rule::string_contents => panic!(),
        Rule::char => panic!(),
        Rule::number_literal => {
            // TODO: integer_literal in grammar??
            Value::Integer(pair.as_str().parse::<i64>().unwrap())
        }
    }
}

fn native_call(name: &str, args: Vec<Value>) -> Value {
    match name {
        "add" => native_call_add(&args),
        "print" => native_call_print(&args, false),
        "println" => native_call_print(&args, true),
        _ => panic!("native function '{}' not found", name),
    }
}

fn native_call_add(args: &[Value]) -> Value {
    let sum = args
        .iter()
        .map(|v| {
            if let Value::Integer(int) = v {
                int
            } else {
                panic!("add: arg is not a number: {:?}", v)
            }
        })
        .sum();

    Value::Integer(sum)
}

fn native_call_print(args: &[Value], newline: bool) -> Value {
    for v in args {
        if let Value::String(s) = v {
            print!("{}", s)
        } else {
            panic!("print: arg is not a string: {:?}", v)
        }
    }

    if newline {
        println!();
    }

    Value::Null
}
