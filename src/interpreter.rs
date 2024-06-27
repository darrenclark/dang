use std::{collections::HashMap, fmt};

use crate::ast::{Node, NodeKind, Source};

#[derive(Debug)]
pub struct Exception {
    source: Option<Source>,
    message: String,
}

impl fmt::Display for Exception {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.source {
            Some(source) => write!(
                f,
                "exception at {}:{}:{}: {}",
                source.file, source.line, source.col, self.message
            ),
            None => write!(f, "exception: {}", self.message),
        }
    }
}

macro_rules! exception {
    ($($arg:tt)*) => {
        return Err(Exception { source: None, message: format!($($arg)*) })
    };
}

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
            context: Context::new(false),
        }
    }

    pub fn new_repl() -> Interpreter {
        Interpreter {
            context: Context::new(true),
        }
    }

    pub fn eval(&mut self, node: &Node) -> Result<Value, Exception> {
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
    /// Can variables be redefined?  Used in REPL for ergonomics.
    allow_redefinition: bool,
}

impl Context {
    fn new(allow_redefinition: bool) -> Context {
        let mut c = Context {
            variables: HashMap::new(),
            allow_redefinition,
        };

        let _ = c.define("add", false, Value::NativeFunc("add"));
        let _ = c.define("print", false, Value::NativeFunc("print"));
        let _ = c.define("println", false, Value::NativeFunc("println"));

        c
    }

    fn get(&self, name: &str) -> Option<&Value> {
        self.variables.get(name).map(|v| &v.value)
    }

    fn define(&mut self, name: &str, mutable: bool, value: Value) -> Result<(), Exception> {
        if self.variables.contains_key(name) && !self.allow_redefinition {
            exception!("variable '{}' already defined", name)
        }
        self.variables
            .insert(String::from(name), Variable { value, mutable });
        Ok(())
    }

    fn assign(&mut self, name: &str, value: Value) -> Result<(), Exception> {
        if let Some(v) = self.variables.get_mut(name) {
            if !v.mutable {
                exception!("variable '{}' is not mutable", name)
            }
            v.value = value;
            Ok(())
        } else {
            exception!("variable '{}' is not defined", name)
        }
    }
}

fn eval(context: &mut Context, node: &Node) -> Result<Value, Exception> {
    do_eval(context, node).map_err(|mut exception| {
        if exception.source.is_none() {
            exception.source = Some(node.source.clone())
        }
        exception
    })
}

fn do_eval(context: &mut Context, node: &Node) -> Result<Value, Exception> {
    match &node.kind {
        NodeKind::SourceFile(children) => {
            let mut result = Value::Null;
            for n in children {
                result = eval(context, n)?
            }
            Ok(result)
        }
        NodeKind::Let { identifier, expr } => match &identifier.kind {
            NodeKind::Identifier(name) => {
                let value = eval(context, expr)?;
                context.define(name, false, value.clone())?;
                Ok(value)
            }
            _ => panic!(),
        },
        NodeKind::Var { identifier, expr } => match &identifier.kind {
            NodeKind::Identifier(name) => {
                let value = eval(context, expr)?;
                context.define(name, true, value.clone())?;
                Ok(value)
            }
            _ => panic!(),
        },
        NodeKind::Assignment { identifier, expr } => match &identifier.kind {
            NodeKind::Identifier(name) => {
                let value = eval(context, expr)?;
                context.assign(name, value.clone())?;
                Ok(value)
            }
            _ => panic!(),
        },
        NodeKind::FunctionCall { function, args } => {
            let func = eval(context, function.as_ref())?;
            if let Value::NativeFunc(name) = func {
                let mut evaled_args = Vec::with_capacity(args.len());
                for n in args {
                    evaled_args.push(eval(context, n)?)
                }
                native_call(name, evaled_args)
            } else {
                exception!("tried to call a non-function value: {:?}", func)
            }
        }
        NodeKind::Identifier(name) => match context.get(name) {
            Some(v) => Ok(v.clone()),
            None => exception!("no binding {}", name),
        },
        NodeKind::StringLiteral(contents) => Ok(Value::String(contents.to_owned())),
        NodeKind::IntegerLiteral(i) => Ok(Value::Integer(*i)),
    }
}

fn native_call(name: &str, args: Vec<Value>) -> Result<Value, Exception> {
    match name {
        "add" => native_call_add(&args),
        "print" => native_call_print(&args, false),
        "println" => native_call_print(&args, true),
        _ => exception!("native function '{}' not found", name),
    }
}

fn native_call_add(args: &[Value]) -> Result<Value, Exception> {
    let sum = args.iter().try_fold(0, |acc, v| {
        if let Value::Integer(int) = v {
            Ok(acc + int)
        } else {
            exception!("add: arg is not a number: {:?}", v)
        }
    })?;

    Ok(Value::Integer(sum))
}

fn native_call_print(args: &[Value], newline: bool) -> Result<Value, Exception> {
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

    Ok(Value::Null)
}
