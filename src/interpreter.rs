use std::collections::HashMap;

use crate::{ast::Node, ast::NodeKind};

#[derive(Debug, Clone)]
pub enum Value {
    Null,
    String(String),
    Integer(i64),
    NativeFunc(&'static str),
}

#[derive(Debug)]
pub struct Interpreter {
    context: Context,
}

impl Interpreter {
    pub fn new() -> Interpreter {
        Interpreter {
            context: Context::new(),
        }
    }

    pub fn eval(&mut self, node: &Node) -> Value {
        eval(&mut self.context, node)
    }
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

fn eval(context: &mut Context, node: &Node) -> Value {
    match &node.kind {
        NodeKind::SourceFile(children) => {
            let mut result = Value::Null;
            for n in children {
                result = eval(context, n);
            }
            result
        }
        NodeKind::Let { identifier, expr } => match &identifier.kind {
            NodeKind::Identifier(name) => {
                let value = eval(context, expr);
                context.put(name, value.clone());
                value
            }
            _ => panic!(),
        },
        NodeKind::FunctionCall { function, args } => {
            let func = eval(context, function.as_ref());
            if let Value::NativeFunc(name) = func {
                let evaled_args = args.iter().map(|n| eval(context, n)).collect();
                native_call(name, evaled_args)
            } else {
                panic!("tried to call a non-function value: {:?}", func)
            }
        }
        NodeKind::Identifier(name) => match context.get(name) {
            Some(v) => v.clone(),
            None => panic!("no binding {}", name),
        },
        NodeKind::StringLiteral(contents) => Value::String(contents.to_owned()),
        NodeKind::IntegerLiteral(i) => Value::Integer(*i),
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
