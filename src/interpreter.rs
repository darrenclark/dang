use std::{
    cell::RefCell,
    collections::{BTreeMap, HashMap},
    fmt,
    mem::discriminant,
    rc::Rc,
};

use crate::{
    ast::{BinOp, Node, NodeKind, Source, UnaryOp},
    compiler::Compiler,
    native_funcs,
    program::Program,
    value::{FunctionLiteral, Value},
};

#[derive(Debug, Clone)]
pub struct Exception {
    pub source: Option<Source>,
    pub message: String,
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
    program: Program,
    globals: Rc<RefCell<Environment>>,
    environment: Rc<RefCell<Environment>>,
    compiler: Compiler,
}

impl Interpreter {
    pub fn new() -> Interpreter {
        let globals = Rc::new(RefCell::new(Environment::new(false)));
        let mut i = Interpreter {
            program: Program::default(),
            globals: globals.clone(),
            environment: globals,
            compiler: Compiler::default(),
        };
        i.load_stdlib();
        i
    }

    pub fn new_repl() -> Interpreter {
        let globals = Rc::new(RefCell::new(Environment::new(true)));
        let mut i = Interpreter {
            program: Program::default(),
            globals: globals.clone(),
            environment: globals,
            compiler: Compiler::default(),
        };
        i.load_stdlib();
        i
    }

    fn load_stdlib(&mut self) {
        for (name, ptr) in native_funcs::funcs() {
            let _ = self
                .globals
                .borrow_mut()
                .define(name, false, Value::NativeFunc(ptr));
        }

        // Load Std
        match self.compiler.compile("Std", &mut self.program) {
            Ok(_) => {}
            Err(errors) => {
                panic!("Failed to load stdlib: {:?}", errors)
            }
        }

        for id in self.program.modules.module_ids() {
            let module = self.program.modules.get_by_id(id);
            if let Err(exception) = self.eval(module.ast.as_ref()) {
                panic!("exception while loading standard library: {}", exception)
            }
        }
    }

    pub fn set_argv(&mut self, argv: Vec<String>) {
        let converted: Vec<Value> = argv.iter().map(|s| Value::String(s.clone())).collect();
        self.globals
            .borrow_mut()
            .define("argv", false, Value::List(converted))
            .unwrap();
    }

    fn switch_to_new_env(&mut self) -> Rc<RefCell<Environment>> {
        let prev_env = self.environment.clone();
        self.environment = Rc::new(RefCell::new(Environment::new_child(prev_env.clone())));
        prev_env
    }

    fn switch_to_env(&mut self, env: Rc<RefCell<Environment>>) -> Rc<RefCell<Environment>> {
        let prev_env = self.environment.clone();
        self.environment = env;
        prev_env
    }

    fn restore_env(&mut self, env: Rc<RefCell<Environment>>) {
        self.environment = env
    }

    pub fn eval(&mut self, node: &Node) -> Result<Value, Exception> {
        self.do_eval(node).map_err(|mut exception| {
            if exception.source.is_none() {
                exception.source = Some(node.source.clone())
            }
            exception
        })
    }

    fn do_eval(&mut self, node: &Node) -> Result<Value, Exception> {
        match &node.kind {
            NodeKind::SourceFile(children) => {
                let mut result = Value::Nil;
                for n in children {
                    result = self.eval(n)?
                }
                Ok(result)
            }
            NodeKind::Module(_) => Ok(Value::Nil),
            NodeKind::Import(_) => Ok(Value::Nil),
            NodeKind::Body(children) => {
                let prev_env = self.switch_to_new_env();
                let mut result = Value::Nil;
                for n in children {
                    result = match self.eval(n) {
                        Ok(r) => r,
                        err => {
                            self.restore_env(prev_env);
                            return err;
                        }
                    }
                }
                self.restore_env(prev_env);
                Ok(result)
            }
            NodeKind::Let { identifier, expr } => match &identifier.kind {
                NodeKind::Identifier(name) => {
                    let value = self.eval(expr)?;
                    self.environment
                        .borrow_mut()
                        .define(name, false, value.clone())?;
                    Ok(value)
                }
                _ => panic!(),
            },
            NodeKind::Var { identifier, expr } => match &identifier.kind {
                NodeKind::Identifier(name) => {
                    let value = self.eval(expr)?;
                    self.environment
                        .borrow_mut()
                        .define(name, true, value.clone())?;
                    Ok(value)
                }
                _ => panic!(),
            },
            NodeKind::Assignment { identifier, expr } => match &identifier.kind {
                NodeKind::Identifier(name) => {
                    let value = self.eval(expr)?;
                    self.environment.borrow_mut().assign(name, value.clone())?;
                    Ok(value)
                }
                _ => panic!(),
            },
            NodeKind::If {
                condition,
                body,
                else_branch,
            } => {
                let c = self.eval(condition)?;
                // TODO: This should push a new context
                if c.truthy() {
                    self.eval(body)
                } else if let Some(else_branch) = else_branch {
                    self.eval(else_branch)
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

                let e = self.eval(enumerable)?.ensure_enumerable("for")?;
                let env = self.switch_to_new_env();
                let res = (|| {
                    self.environment
                        .borrow_mut()
                        .define(var_name, true, Value::Nil)?;
                    for i in 0..e.enum_len() {
                        self.environment
                            .borrow_mut()
                            .assign(var_name, e.enum_at(i).unwrap())?;
                        self.eval(body)?;
                    }

                    Ok(Value::Nil)
                })();
                self.restore_env(env);
                res
            }
            NodeKind::Subscript { object, key } => {
                let object = self.eval(object)?;
                let key = self.eval(key)?;
                object.at(&key)
            }
            NodeKind::FieldAccess { object, key } => {
                let object = self.eval(object)?;
                let key = match &key.kind {
                    NodeKind::Identifier(name) => name,
                    _ => unreachable!(),
                };
                object.at_field(key)
            }
            NodeKind::FunctionCall { function, args } => {
                let func = self.eval(function.as_ref())?;
                if let Value::NativeFunc(ptr) = func {
                    let mut evaled_args = Vec::with_capacity(args.len());
                    for n in args {
                        evaled_args.push(self.eval(n)?)
                    }
                    ptr(&evaled_args)
                } else if let Value::Func(function_literal) = func {
                    let mut evaled_args = Vec::with_capacity(args.len());
                    for n in args {
                        evaled_args.push(self.eval(n)?)
                    }
                    self.call(&function_literal, &evaled_args)
                } else {
                    exception!("tried to call a non-function value: {:?}", func)
                }
            }
            NodeKind::VariableRef { identifier } => match self
                .environment
                .borrow()
                .get(identifier.unwrap_identifier())
            {
                Some(v) => Ok(v.clone()),
                None => exception!("no binding {}", identifier.unwrap_identifier()),
            },
            NodeKind::Identifier(_) => unreachable!(),
            NodeKind::ListLiteral(elements) => {
                let mut evaled_elements = Vec::with_capacity(elements.len());
                for e in elements {
                    evaled_elements.push(self.eval(e)?);
                }
                Ok(Value::List(evaled_elements))
            }
            NodeKind::DictLiteral(key_values) => {
                let mut map = BTreeMap::new();
                for (k, v) in key_values {
                    map.insert(self.eval(k)?, self.eval(v)?);
                }
                Ok(Value::Dict(map))
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
                let evaled_lhs = self.eval(lhs)?;
                if evaled_lhs.truthy() {
                    Ok(evaled_lhs)
                } else {
                    self.eval(rhs)
                }
            }
            NodeKind::BinaryOp {
                op: BinOp::LogicalAnd,
                lhs,
                rhs,
            } => {
                let evaled_lhs = self.eval(lhs)?;
                if evaled_lhs.truthy() {
                    self.eval(rhs)
                } else {
                    Ok(evaled_lhs)
                }
            }
            NodeKind::BinaryOp { op, lhs, rhs } => bin_op(*op, self.eval(lhs)?, self.eval(rhs)?),
            NodeKind::UnaryOp { op, rhs } => unary_op(*op, self.eval(rhs)?),
            NodeKind::FunctionLiteral {
                arg_names: _,
                body: _,
            } => {
                let body = Rc::new(node.clone());
                let environment = self.environment.clone();
                let func_literal = FunctionLiteral { body, environment };
                Ok(Value::Func(func_literal))
            }
            NodeKind::Pipe { lhs, rhs } => {
                let lhs = self.eval(lhs)?;

                let (func, mut rest_args) = match &rhs.kind {
                    NodeKind::FunctionCall { function, args } => {
                        let mut evaled_args = Vec::with_capacity(args.len());
                        for n in args {
                            evaled_args.push(self.eval(n)?)
                        }
                        (self.eval(function.as_ref())?, evaled_args)
                    }
                    _ => exception!("not piping in to a function"),
                };

                let mut args = vec![lhs];
                args.append(&mut rest_args);

                if let Value::NativeFunc(ptr) = func {
                    ptr(&args)
                } else if let Value::Func(function_literal) = func {
                    self.call(&function_literal, &args)
                } else {
                    exception!("tried to call a non-function value: {:?}", func)
                }
            }
        }
    }

    fn call(
        &mut self,
        function_literal: &FunctionLiteral,
        args: &[Value],
    ) -> Result<Value, Exception> {
        // push env for closures
        let prev_env = self.switch_to_env(function_literal.environment.clone());
        // push an empty env for this call's args & locals
        let _ = self.switch_to_new_env();
        let result = self.inner_call(function_literal.body.as_ref(), args);
        self.restore_env(prev_env);
        result
    }

    fn inner_call(&mut self, function_literal: &Node, args: &[Value]) -> Result<Value, Exception> {
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
            self.environment
                .borrow_mut()
                .define(name, false, value.clone())?;
        }

        self.eval(body)
    }
}

#[derive(Debug)]
struct Variable {
    value: Value,
    mutable: bool,
}

#[derive(Debug)]
pub struct Environment {
    parent: Option<Rc<RefCell<Environment>>>,
    /// Variables
    variables: HashMap<String, Variable>,
    /// Can variables be redefined?  Used in REPL for ergonomics.
    allow_redefinition: bool,
}

impl Environment {
    fn new(allow_redefinition: bool) -> Environment {
        Environment {
            parent: None,
            variables: HashMap::new(),
            allow_redefinition,
        }
    }

    fn new_child(parent: Rc<RefCell<Environment>>) -> Environment {
        Environment {
            parent: Some(parent),
            variables: HashMap::new(),
            allow_redefinition: false,
        }
    }

    fn get(&self, name: &str) -> Option<Value> {
        self.variables
            .get(name)
            .map(|v| v.value.clone())
            .or_else(|| {
                if let Some(parent) = &self.parent {
                    parent.borrow().get(name)
                } else {
                    None
                }
            })
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
        } else if let Some(parent) = &self.parent {
            parent.borrow_mut().assign(name, value)
        } else {
            exception!("variable '{}' is not defined", name)
        }
    }
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
