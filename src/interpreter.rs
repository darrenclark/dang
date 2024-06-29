use std::{collections::HashMap, fmt, rc::Rc};

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
    Bool(bool),
    String(String),
    Integer(i64),
    NativeFunc(&'static str),
    Func { function_literal: Rc<Node> },
}
impl Value {
    fn truthy(&self) -> bool {
        match self {
            Self::Null => false,
            Self::Bool(v) => *v,
            _ => true,
        }
    }
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
    /// Stack frames.  Frame at 0 is also global scope.
    frames: Vec<Frame>,
    /// Can variables be redefined?  Used in REPL for ergonomics.
    allow_redefinition: bool,
}

#[derive(Debug)]
struct Frame {
    variables: HashMap<String, Variable>,
}

impl Frame {
    fn new() -> Frame {
        Frame {
            variables: HashMap::new(),
        }
    }
}

impl Context {
    fn new(allow_redefinition: bool) -> Context {
        let mut c = Context {
            frames: vec![Frame::new()],
            allow_redefinition,
        };

        let _ = c.define("add", false, Value::NativeFunc("add"));
        let _ = c.define("print", false, Value::NativeFunc("print"));
        let _ = c.define("println", false, Value::NativeFunc("println"));

        c
    }

    fn get(&self, name: &str) -> Option<&Value> {
        self.current_frame()
            .variables
            .get(name)
            .or_else(|| self.global_frame().variables.get(name))
            .map(|v| &v.value)
    }

    fn define(&mut self, name: &str, mutable: bool, value: Value) -> Result<(), Exception> {
        if self.current_frame().variables.contains_key(name) && !self.allow_redefinition {
            exception!("variable '{}' already defined", name)
        }
        self.current_frame_mut()
            .variables
            .insert(String::from(name), Variable { value, mutable });
        Ok(())
    }

    fn assign(&mut self, name: &str, value: Value) -> Result<(), Exception> {
        if let Some(v) = self.current_frame_mut().variables.get_mut(name) {
            if !v.mutable {
                exception!("variable '{}' is not mutable", name)
            }
            v.value = value;
            Ok(())
        } else if let Some(v) = self.global_frame_mut().variables.get_mut(name) {
            if !v.mutable {
                exception!("variable '{}' is not mutable", name)
            }
            v.value = value;
            Ok(())
        } else {
            exception!("variable '{}' is not defined", name)
        }
    }

    fn global_frame(&self) -> &Frame {
        self.frames.first().unwrap()
    }

    fn global_frame_mut(&mut self) -> &mut Frame {
        self.frames.first_mut().unwrap()
    }

    fn current_frame(&self) -> &Frame {
        self.frames.last().unwrap()
    }

    fn current_frame_mut(&mut self) -> &mut Frame {
        self.frames.last_mut().unwrap()
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
        NodeKind::Body(children) => {
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
        NodeKind::If {
            condition,
            body,
            else_branch,
        } => {
            let c = eval(context, condition)?;
            if c.truthy() {
                eval(context, body)
            } else if let Some(else_branch) = else_branch {
                eval(context, else_branch)
            } else {
                Ok(Value::Null)
            }
        }
        NodeKind::FunctionCall { function, args } => {
            let func = eval(context, function.as_ref())?;
            if let Value::NativeFunc(name) = func {
                let mut evaled_args = Vec::with_capacity(args.len());
                for n in args {
                    evaled_args.push(eval(context, n)?)
                }
                native_call(name, evaled_args)
            } else if let Value::Func { function_literal } = func {
                let mut evaled_args = Vec::with_capacity(args.len());
                for n in args {
                    evaled_args.push(eval(context, n)?)
                }
                call(context, function_literal.as_ref(), &evaled_args)
            } else {
                exception!("tried to call a non-function value: {:?}", func)
            }
        }
        NodeKind::Identifier(name) => match context.get(name) {
            Some(v) => Ok(v.clone()),
            None => exception!("no binding {}", name),
        },
        NodeKind::BoolLiteral(v) => Ok(Value::Bool(*v)),
        NodeKind::StringLiteral(contents) => Ok(Value::String(contents.to_owned())),
        NodeKind::IntegerLiteral(i) => Ok(Value::Integer(*i)),
        NodeKind::Add { lhs, rhs } => add_values(eval(context, lhs)?, eval(context, rhs)?),
        NodeKind::Sub { lhs, rhs } => sub_values(eval(context, lhs)?, eval(context, rhs)?),
        NodeKind::Mul { lhs, rhs } => mul_values(eval(context, lhs)?, eval(context, rhs)?),
        NodeKind::Div { lhs, rhs } => div_values(eval(context, lhs)?, eval(context, rhs)?),
        NodeKind::Negate { rhs } => negate_value(eval(context, rhs)?),
        NodeKind::FunctionLiteral {
            arg_names: _,
            body: _,
        } => Ok(Value::Func {
            function_literal: Rc::new(node.clone()),
        }),
    }
}

fn call(
    context: &mut Context,
    function_literal: &Node,
    args: &[Value],
) -> Result<Value, Exception> {
    context.frames.push(Frame::new());
    let result = inner_call(context, function_literal, args);
    context.frames.pop();
    result
}

fn inner_call(
    context: &mut Context,
    function_literal: &Node,
    args: &[Value],
) -> Result<Value, Exception> {
    let (arg_names, body) = match &function_literal.kind {
        NodeKind::FunctionLiteral { arg_names, body } => (arg_names, body),
        _ => panic!(),
    };

    if arg_names.len() != args.len() {
        exception!("expected {} args, got {}", arg_names.len(), args.len())
    }

    for (arg_name, value) in arg_names.iter().zip(args.iter()) {
        let name = match &arg_name.kind {
            NodeKind::Identifier(name) => name,
            _ => panic!(),
        };
        context.define(name, false, value.clone())?;
    }

    let mut result = Value::Null;

    for n in body {
        result = eval(context, n)?;
    }

    Ok(result)
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

fn add_values(lhs: Value, rhs: Value) -> Result<Value, Exception> {
    match (&lhs, &rhs) {
        (Value::String(l), Value::String(r)) => Ok(Value::String(l.to_owned() + r)),
        (Value::Integer(l), Value::Integer(r)) => Ok(Value::Integer(l + r)),
        (l, r) => exception!("cannot apply `+` to {:?} and {:?}", l, r),
    }
}

fn sub_values(lhs: Value, rhs: Value) -> Result<Value, Exception> {
    match (&lhs, &rhs) {
        (Value::Integer(l), Value::Integer(r)) => Ok(Value::Integer(l - r)),
        (l, r) => exception!("cannot apply `+` to {:?} and {:?}", l, r),
    }
}

fn mul_values(lhs: Value, rhs: Value) -> Result<Value, Exception> {
    match (&lhs, &rhs) {
        (Value::Integer(l), Value::Integer(r)) => Ok(Value::Integer(l * r)),
        (l, r) => exception!("cannot apply `+` to {:?} and {:?}", l, r),
    }
}

fn div_values(lhs: Value, rhs: Value) -> Result<Value, Exception> {
    match (&lhs, &rhs) {
        (Value::Integer(l), Value::Integer(r)) => Ok(Value::Integer(l / r)),
        (l, r) => exception!("cannot apply `+` to {:?} and {:?}", l, r),
    }
}

fn negate_value(rhs: Value) -> Result<Value, Exception> {
    match &rhs {
        Value::Integer(r) => Ok(Value::Integer(-r)),
        v => exception!("cannot negate (`-`) {:?}", v),
    }
}
