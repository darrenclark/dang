use std::fmt;
use std::hash::Hash;
use std::rc::Rc;

use super::chunk::Chunk;

#[derive(Clone)]
pub struct Function {
    inner: Rc<FunctionInner>,
}

impl Function {
    pub fn new(chunk: Chunk) -> Self {
        Self {
            inner: Rc::new(FunctionInner { chunk }),
        }
    }

    pub fn chunk(&self) -> &Chunk {
        &self.inner.chunk
    }
}

impl fmt::Debug for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Function<{:p}>", self.inner)
    }
}

impl Eq for Function {}

impl PartialEq for Function {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.inner, &other.inner)
    }
}

impl Hash for Function {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        Rc::as_ptr(&self.inner).hash(state)
    }
}

impl Ord for Function {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let self_ptr = Rc::as_ptr(&self.inner);
        let other_ptr = Rc::as_ptr(&other.inner);
        self_ptr.cmp(&other_ptr)
    }
}

impl PartialOrd for Function {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Clone)]
struct FunctionInner {
    chunk: Chunk,
}
