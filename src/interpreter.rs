use std::{collections::HashMap, fmt, mem::discriminant, rc::Rc};

use crate::{
    ast::{BinOp, Node, NodeKind, Source, UnaryOp},
    native_funcs,
    stdlib::load_stdlib,
    value::{FunctionLiteralRc, Value},
};

#[derive(Debug)]
pub struct Exception {
    source: Option<Source>,
    message: String,
}

impl Exception {
    pub fn new(message: String) -> Exception {
        Exception {
            source: None,
            message,
        }
    }
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
        return Err(Exception::new(format!($($arg)*)))
    };
}
pub(crate) use exception;

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

    pub fn set_argv(&mut self, argv: Vec<String>) {
        let converted: Vec<Value> = argv.iter().map(|s| Value::String(s.clone())).collect();
        self.context
            .define("argv", false, Value::List(converted))
            .unwrap();
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

        for (name, ptr) in native_funcs::funcs() {
            let _ = c.define(name, false, Value::NativeFunc(ptr));
        }

        for module in load_stdlib() {
            if let Err(exception) = eval(&mut c, &module) {
                panic!("exception while loading standard library: {}", exception)
            }
        }

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
            let mut result = Value::Nil;
            for n in children {
                result = eval(context, n)?
            }
            Ok(result)
        }
        NodeKind::Body(children) => {
            let mut result = Value::Nil;
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
            // TODO: This should push a new context
            if c.truthy() {
                eval(context, body)
            } else if let Some(else_branch) = else_branch {
                eval(context, else_branch)
            } else {
                Ok(Value::Nil)
            }
        }
        NodeKind::For {
            var_name,
            enumerable,
            body,
        } => {
            let var_name = match &var_name.kind {
                NodeKind::Identifier(name) => name,
                _ => panic!(),
            };

            let e = eval(context, enumerable)?.ensure_enumerable("for")?;
            // TODO: This should push a new context
            context.define(var_name, true, Value::Nil)?;
            for i in 0..e.enum_len() {
                context.assign(var_name, e.enum_at(i).unwrap())?;
                eval(context, body)?;
            }

            Ok(Value::Nil)
        }
        NodeKind::Subscript { object, key } => {
            let object = eval(context, object)?;
            let key = eval(context, key)?;
            object.ensure_enumerable("subscript")?;
            object.enum_at(key.to_index()?)
        }
        NodeKind::FunctionCall { function, args } => {
            let func = eval(context, function.as_ref())?;
            if let Value::NativeFunc(ptr) = func {
                let mut evaled_args = Vec::with_capacity(args.len());
                for n in args {
                    evaled_args.push(eval(context, n)?)
                }
                ptr(&evaled_args)
            } else if let Value::Func(function_literal_rc) = func {
                let mut evaled_args = Vec::with_capacity(args.len());
                for n in args {
                    evaled_args.push(eval(context, n)?)
                }
                call(context, function_literal_rc.0.as_ref(), &evaled_args)
            } else {
                exception!("tried to call a non-function value: {:?}", func)
            }
        }
        NodeKind::Identifier(name) => match context.get(name) {
            Some(v) => Ok(v.clone()),
            None => exception!("no binding {}", name),
        },
        NodeKind::ListLiteral(elements) => {
            let mut evaled_elements = Vec::with_capacity(elements.len());
            for e in elements {
                evaled_elements.push(eval(context, e)?);
            }
            Ok(Value::List(evaled_elements))
        }
        NodeKind::NilLiteral => Ok(Value::Nil),
        NodeKind::BoolLiteral(v) => Ok(Value::Bool(*v)),
        NodeKind::StringLiteral(contents) => Ok(Value::String(contents.to_owned())),
        NodeKind::IntegerLiteral(i) => Ok(Value::Integer(*i)),
        NodeKind::BinaryOp {
            op: BinOp::LogicalOr,
            lhs,
            rhs,
        } => {
            let evaled_lhs = eval(context, lhs)?;
            if evaled_lhs.truthy() {
                Ok(evaled_lhs)
            } else {
                eval(context, rhs)
            }
        }
        NodeKind::BinaryOp {
            op: BinOp::LogicalAnd,
            lhs,
            rhs,
        } => {
            let evaled_lhs = eval(context, lhs)?;
            if evaled_lhs.truthy() {
                eval(context, rhs)
            } else {
                Ok(evaled_lhs)
            }
        }
        NodeKind::BinaryOp { op, lhs, rhs } => {
            bin_op(*op, eval(context, lhs)?, eval(context, rhs)?)
        }
        NodeKind::UnaryOp { op, rhs } => unary_op(*op, eval(context, rhs)?),
        NodeKind::FunctionLiteral {
            arg_names: _,
            body: _,
        } => Ok(Value::Func(FunctionLiteralRc(Rc::new(node.clone())))),
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

    let mut result = Value::Nil;

    for n in body {
        result = eval(context, n)?;
    }

    Ok(result)
}

fn bin_op(op: BinOp, lhs: Value, rhs: Value) -> Result<Value, Exception> {
    match (op, &lhs, &rhs) {
        // Addition
        (BinOp::Add, Value::String(l), Value::String(r)) => Ok(Value::String(l.to_owned() + r)),
        (BinOp::Add, Value::Integer(l), Value::Integer(r)) => Ok(Value::Integer(l + r)),
        (BinOp::Add, Value::List(l), Value::List(r)) => {
            let mut res = l.to_owned();
            res.extend(r.iter().cloned());
            Ok(Value::List(res))
        }
        // Subtraction
        (BinOp::Sub, Value::Integer(l), Value::Integer(r)) => Ok(Value::Integer(l - r)),
        // Multiplication
        (BinOp::Mul, Value::Integer(l), Value::Integer(r)) => Ok(Value::Integer(l * r)),
        // Division
        (BinOp::Div, Value::Integer(l), Value::Integer(r)) => Ok(Value::Integer(l / r)),
        // || &&
        (BinOp::LogicalOr | BinOp::LogicalAnd, _, _) => unreachable!("handled in caller"),
        // == / !=
        (BinOp::Eq, l, r) => Ok(Value::Bool(l == r)),
        (BinOp::Neq, l, r) => Ok(Value::Bool(l == r)),
        // >=, <, etc.
        (BinOp::Gt, l, r) if discriminant(l) == discriminant(r) => Ok(Value::Bool(l > r)),
        (BinOp::Gte, l, r) if discriminant(l) == discriminant(r) => Ok(Value::Bool(l >= r)),
        (BinOp::Lt, l, r) if discriminant(l) == discriminant(r) => Ok(Value::Bool(l < r)),
        (BinOp::Lte, l, r) if discriminant(l) == discriminant(r) => Ok(Value::Bool(l <= r)),
        // Error
        (_, l, r) => {
            exception!("cannot apply `{:?}` to {:?} and {:?}", op, l, r)
        }
    }
}

fn unary_op(op: UnaryOp, rhs: Value) -> Result<Value, Exception> {
    match (op, &rhs) {
        (UnaryOp::Neg, Value::Integer(r)) => Ok(Value::Integer(-r)),
        (UnaryOp::LogicalNeg, v) => Ok(Value::Bool(!v.truthy())),
        (_, v) => exception!("cannot {:?} {:?}", op, v),
    }
}
