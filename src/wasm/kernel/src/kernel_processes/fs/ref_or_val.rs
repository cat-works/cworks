use std::ops::{Deref, DerefMut};

#[derive(Clone)]
pub enum RefOrVal<T> {
    Val(T),
    Ref(Box<T>),
}

impl<T: std::fmt::Display> std::fmt::Display for RefOrVal<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RefOrVal::Val(x) => write!(f, "{x}"),
            RefOrVal::Ref(x) => write!(f, "&{}", *x),
        }
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for RefOrVal<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RefOrVal::Val(x) => write!(f, "{x:?}"),
            RefOrVal::Ref(x) => write!(f, "&{:?}", *x),
        }
    }
}

impl<T> Deref for RefOrVal<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        match self {
            RefOrVal::Val(x) => x,
            RefOrVal::Ref(x) => (*x).as_ref(),
        }
    }
}
impl<T> DerefMut for RefOrVal<T> {
    fn deref_mut(&mut self) -> &mut T {
        match self {
            RefOrVal::Val(x) => x,
            RefOrVal::Ref(x) => (*x).as_mut(),
        }
    }
}
