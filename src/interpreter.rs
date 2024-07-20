use std::{
    cell::RefCell,
    collections::{BTreeMap, HashMap, HashSet},
    fmt,
    mem::discriminant,
    rc::Rc,
};

use crate::{
    ast::{BinOp, Node, NodeKind, Source, UnaryOp},
    compiler::{Compiler, Input},
    module::ModuleName,
    native_funcs,
    program::Program,
    scope::VariableLocation,
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
use ustr::Ustr;

#[derive(Debug)]
pub struct Interpreter {
    program: Program,
    globals: Rc<RefCell<Environment>>,
    environment: Rc<RefCell<Environment>>,
    pub compiler: Compiler,
    loaded_modules: HashSet<ModuleName>,
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

impl Interpreter {
    pub fn new() -> Interpreter {
        let globals = Rc::new(RefCell::new(Environment::new(false)));
        Interpreter {
            program: Program::default(),
            globals: globals.clone(),
            environment: globals,
            compiler: Compiler::default(),
            loaded_modules: HashSet::default(),
        }
    }

    pub fn new_repl() -> Interpreter {
        let globals = Rc::new(RefCell::new(Environment::new(true)));
        Interpreter {
            program: Program::default(),
            globals: globals.clone(),
            environment: globals,
            compiler: Compiler::default(),
            loaded_modules: HashSet::default(),
        }
    }

    pub fn set_argv(&mut self, argv: Vec<String>) {
        self.run(Input::ModuleName("Std/Builtins".to_owned()))
            .expect("Std/Builtins failed to load");

        let converted: Vec<Value> = argv.iter().map(|s| Value::String(s.clone())).collect();

        Environment::assign(
            self.globals.clone(),
            &VariableLocation::Global {
                module: "Std/Builtins".into(),
                name: "argv".into(),
            },
            Value::List(converted),
        )
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

    pub fn run(&mut self, input: Input) -> Result<Value, Exception> {
        match self.compiler.compile(input, &mut self.program) {
            Ok(_) => {}
            Err(errors) => {
                exception!("{:?}", errors)
            }
        }

        let mut value = Value::Nil;
        for name in self.program.module_names() {
            if self.loaded_modules.contains(&name) {
                continue;
            }

            self.loaded_modules.insert(name);
            let module = self.program.get_module(&name).unwrap();
            value = self.eval(module.ast.as_ref())?
        }
        Ok(value)
    }

    fn eval(&mut self, node: &Node) -> Result<Value, Exception> {
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
            NodeKind::Builtin { identifier } => match &identifier.kind {
                NodeKind::Identifier(name) => {
                    let value = Value::NativeFunc(native_funcs::func(name).unwrap());
                    self.define(node, false, value.clone())?;
                    Ok(value)
                }
                _ => panic!(),
            },
            NodeKind::Let { identifier, expr } => match &identifier.kind {
                NodeKind::Identifier(_) => {
                    let value = self.eval(expr)?;
                    self.define(node, false, value.clone())?;
                    Ok(value)
                }
                _ => panic!(),
            },
            NodeKind::Var { identifier, expr } => match &identifier.kind {
                NodeKind::Identifier(_) => {
                    let value = self.eval(expr)?;
                    self.define(node, true, value.clone())?;
                    Ok(value)
                }
                _ => panic!(),
            },
            NodeKind::Assignment { identifier, expr } => match &identifier.kind {
                NodeKind::Identifier(_) => {
                    let value = self.eval(expr)?;
                    Environment::assign(
                        self.environment.clone(),
                        &self.variable_location(node),
                        value.clone(),
                    )?;
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
                let env = self.switch_to_new_env();
                let e = self.eval(enumerable)?.ensure_enumerable("for")?;
                let res = (|| {
                    let var_name = Ustr::from(var_name.unwrap_identifier());
                    self.define(node, true, Value::Nil)?;
                    let location = VariableLocation::Closure {
                        name: var_name,
                        nth_parent: 0,
                    };
                    for i in 0..e.enum_len() {
                        Environment::assign(
                            self.environment.clone(),
                            &location,
                            e.enum_at(i).unwrap(),
                        )?;
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
                match object {
                    Value::Symbol(module_name) => {
                        let key = match key {
                            Value::String(k) => k,
                            _ => exception!("string required when subscripting a module"),
                        };
                        self.module_field_access(&module_name, &key)
                    }
                    value => value.at(&key),
                }
            }
            NodeKind::FieldAccess { object, key } => {
                let object = self.eval(object)?;
                match object {
                    Value::Symbol(module_name) => {
                        self.module_field_access(&module_name, key.unwrap_identifier())
                    }
                    value => value.at_field(key.unwrap_identifier()),
                }
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
            NodeKind::VariableRef { identifier } => {
                match Environment::get(self.environment.clone(), &self.variable_location(node)) {
                    Some(v) => Ok(v.clone()),
                    None => {
                        exception!("no binding {}", identifier.unwrap_identifier())
                    }
                }
            }
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
            self.define(arg_name, true, value.clone())?;
        }

        self.eval(body)
    }

    fn define(&mut self, node: &Node, mutable: bool, value: Value) -> Result<(), Exception> {
        let location = self.variable_location(node);
        Environment::define(self.environment.clone(), &location, mutable, value)
    }

    fn variable_location(&self, node: &Node) -> VariableLocation {
        self.program
            .get_module(&node.id.module())
            .expect("expected module loaded")
            .variable_locations
            .get(&node.id)
            .expect("variable location missing for node")
            .clone()
    }

    fn module_field_access(&self, module_name: &str, field_name: &str) -> Result<Value, Exception> {
        let module = match self.program.get_module(&module_name.into()) {
            Some(module) => module,
            None => exception!("module {} not loaded", module_name),
        };
        match module.exports.get(field_name) {
            Some(_) => {
                let location = VariableLocation::Global {
                    module: module.name,
                    name: Ustr::from(field_name),
                };
                match Environment::get(self.globals.clone(), &location) {
                    Some(value) => Ok(value),
                    None => exception!("{}.{} not found", module_name, field_name),
                }
            }
            None => {
                exception!("{}.{} not found", module_name, field_name)
            }
        }
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
    /// Variables - (module, name) => Variable
    variables: HashMap<(Ustr, Ustr), Variable>,
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

    fn root(env: Rc<RefCell<Environment>>) -> Rc<RefCell<Environment>> {
        let mut env = env;

        loop {
            let parent = env.borrow().parent.clone();
            match parent {
                Some(parent) => env = parent,
                None => return env,
            }
        }
    }

    fn nth_parent(env: Rc<RefCell<Environment>>, n: usize) -> Rc<RefCell<Environment>> {
        let mut env = env;

        for _ in 0..n {
            let parent = env.borrow().parent.clone();
            env = parent.expect("expected to have at least n parents");
        }

        env
    }

    fn get(env: Rc<RefCell<Environment>>, variable: &VariableLocation) -> Option<Value> {
        match variable {
            VariableLocation::Constant(value) => Some(value.clone()),
            VariableLocation::Global { .. } => Self::root(env)
                .borrow()
                .variables
                .get(&variable.get_key().unwrap())
                .map(|v| v.value.clone()),
            VariableLocation::Closure {
                name: _,
                nth_parent,
            } => Self::nth_parent(env, *nth_parent)
                .borrow()
                .variables
                .get(&variable.get_key().unwrap())
                .map(|v| v.value.clone()),
            VariableLocation::Local { .. } => env
                .borrow()
                .variables
                .get(&variable.get_key().unwrap())
                .map(|v| v.value.clone()),
        }
    }

    fn define(
        env: Rc<RefCell<Environment>>,
        variable: &VariableLocation,
        mutable: bool,
        value: Value,
    ) -> Result<(), Exception> {
        let env = match variable {
            VariableLocation::Constant(_) => {
                exception!("variable for '{:?}' is not mutable", variable)
            }
            VariableLocation::Global { module: _, name: _ } => Self::root(env),
            VariableLocation::Closure {
                name: _,
                nth_parent,
            } => Self::nth_parent(env, *nth_parent),
            VariableLocation::Local { name: _ } => env,
        };

        if !env.borrow().allow_redefinition
            && env
                .borrow()
                .variables
                .contains_key(&variable.get_key().unwrap())
        {
            exception!(
                "variable for '{:?}' already defined",
                variable.get_name().unwrap()
            )
        } else {
            let v = Variable { value, mutable };
            env.borrow_mut()
                .variables
                .insert(variable.get_key().unwrap(), v);
        }

        Ok(())
    }

    fn assign(
        env: Rc<RefCell<Environment>>,
        variable: &VariableLocation,
        value: Value,
    ) -> Result<(), Exception> {
        let env = match variable {
            VariableLocation::Constant(_) => {
                exception!("variable for '{:?}' is not mutable", variable)
            }
            VariableLocation::Global { module: _, name: _ } => Self::root(env),
            VariableLocation::Closure {
                name: _,
                nth_parent,
            } => Self::nth_parent(env, *nth_parent),
            VariableLocation::Local { name: _ } => env,
        };

        if let Some(v) = env
            .borrow_mut()
            .variables
            .get_mut(&variable.get_key().unwrap())
        {
            if !v.mutable {
                exception!("variable for '{:?}' is not mutable", variable)
            }
            v.value = value;
        } else {
            exception!("variable for '{:?}' is not defined", variable)
        }

        Ok(())
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
