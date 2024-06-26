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
struct Variable {
    value: Value,
    mutable: bool,
}

#[derive(Debug)]
struct Context {
    variables: HashMap<String, Variable>,
}

impl Context {
    fn new() -> Context {
        let mut c = Context {
            variables: HashMap::new(),
        };

        c.define("add", false, Value::NativeFunc("add"));
        c.define("print", false, Value::NativeFunc("print"));
        c.define("println", false, Value::NativeFunc("println"));

        c
    }

    fn get(&self, name: &str) -> Option<&Value> {
        self.variables.get(name).map(|v| &v.value)
    }

    fn define(&mut self, name: &str, mutable: bool, value: Value) {
        if self.variables.contains_key(name) {
            panic!("variable '{}' already defined", name)
        }
        self.variables
            .insert(String::from(name), Variable { value, mutable });
    }

    fn assign(&mut self, name: &str, value: Value) {
        if let Some(v) = self.variables.get_mut(name) {
            if !v.mutable {
                panic!("variable '{}' is not mutable", name)
            }
            v.value = value
        } else {
            panic!("variable '{}' is not defined", name)
        }
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
                context.define(name, false, value.clone());
                value
            }
            _ => panic!(),
        },
        NodeKind::Var { identifier, expr } => match &identifier.kind {
            NodeKind::Identifier(name) => {
                // TODO: immutable vs mutable
                let value = eval(context, expr);
                context.define(name, true, value.clone());
                value
            }
            _ => panic!(),
        },
        NodeKind::Assignment { identifier, expr } => match &identifier.kind {
            NodeKind::Identifier(name) => {
                // TODO: ensure value has been defined before
                let value = eval(context, expr);
                context.assign(name, value.clone());
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
