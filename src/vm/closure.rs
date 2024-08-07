use std::fmt;
use std::hash::Hash;
use std::rc::Rc;

use super::function::Function;
use super::upvalue::Upvalue;

#[derive(Clone)]
pub struct Closure {
    inner: Rc<ClosureInner>,
}

impl Closure {
    pub fn new(function: Function, upvalues: Vec<Upvalue>) -> Self {
        Self {
            inner: Rc::new(ClosureInner { function, upvalues }),
        }
    }

    pub fn function(&self) -> &Function {
        &self.inner.function
    }

    pub fn upvalues(&self) -> &[Upvalue] {
        &self.inner.upvalues
    }
}

impl fmt::Debug for Closure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Closure<{:p} function={:?}>",
            self.inner, self.inner.function
        )
    }
}

impl Eq for Closure {}

impl PartialEq for Closure {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.inner, &other.inner)
    }
}

impl Hash for Closure {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        Rc::as_ptr(&self.inner).hash(state)
    }
}

impl Ord for Closure {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let self_ptr = Rc::as_ptr(&self.inner);
        let other_ptr = Rc::as_ptr(&other.inner);
        self_ptr.cmp(&other_ptr)
    }
}

impl PartialOrd for Closure {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Clone)]
struct ClosureInner {
    function: Function,
    upvalues: Vec<Upvalue>,
}
