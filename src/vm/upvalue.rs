use std::{
    cell::{Ref, RefCell, RefMut},
    ops::{Deref, DerefMut},
    rc::Rc,
};

use crate::value::Value;

#[derive(Debug, Clone)]
pub struct Upvalue {
    inner: Rc<RefCell<UpvalueInner>>,
}

impl Upvalue {
    pub fn new(stack_index: usize) -> Self {
        Self {
            inner: Rc::new(RefCell::new(UpvalueInner::Open { stack_index })),
        }
    }

    pub fn get<'a, 's>(&'a self, stack: &'s [Value]) -> UpvalueRef<'a, 's> {
        let r = self.inner.borrow();
        match *r {
            UpvalueInner::Open { stack_index } => UpvalueRef::OnStack(&stack[stack_index]),
            UpvalueInner::Closed(_) => {
                let r2 = Ref::map(r, |r| match r {
                    UpvalueInner::Open { .. } => unreachable!(),
                    UpvalueInner::Closed(ref value) => value,
                });
                UpvalueRef::InUpvalue(r2)
            }
        }
    }

    pub fn get_mut<'a, 's>(&'a self, stack: &'s mut [Value]) -> UpvalueRefMut<'a, 's> {
        let r = self.inner.borrow_mut();
        match *r {
            UpvalueInner::Open { stack_index } => UpvalueRefMut::OnStack(&mut stack[stack_index]),
            UpvalueInner::Closed(_) => {
                let r2 = RefMut::map(r, |r| match r {
                    UpvalueInner::Open { .. } => unreachable!(),
                    UpvalueInner::Closed(ref mut value) => value,
                });
                UpvalueRefMut::InUpvalue(r2)
            }
        }
    }
}

pub enum UpvalueRef<'a, 's> {
    OnStack(&'s Value),
    InUpvalue(Ref<'a, Value>),
}

impl Deref for UpvalueRef<'_, '_> {
    type Target = Value;
    fn deref(&self) -> &Self::Target {
        match self {
            UpvalueRef::OnStack(value) => value,
            UpvalueRef::InUpvalue(value) => value,
        }
    }
}

pub enum UpvalueRefMut<'a, 's> {
    OnStack(&'s mut Value),
    InUpvalue(RefMut<'a, Value>),
}

impl Deref for UpvalueRefMut<'_, '_> {
    type Target = Value;
    fn deref(&self) -> &Self::Target {
        match self {
            UpvalueRefMut::OnStack(value) => value,
            UpvalueRefMut::InUpvalue(value) => value,
        }
    }
}

impl DerefMut for UpvalueRefMut<'_, '_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            UpvalueRefMut::OnStack(value) => value,
            UpvalueRefMut::InUpvalue(value) => value,
        }
    }
}

#[derive(Debug)]
pub enum UpvalueInner {
    Open { stack_index: usize },
    Closed(Value),
}
