use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};

use crate::{process::SyscallError, Syscall};

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

#[derive(Debug, Clone)]
pub enum FSObj {
    Int(RefOrVal<i128>),
    String(RefOrVal<String>),
    Boolean(RefOrVal<bool>),
    Float(RefOrVal<f32>),
    Double(RefOrVal<f64>),
    Bytes(RefOrVal<Vec<u8>>),
    List(RefOrVal<Vec<FSObj>>),
    Dist(RefOrVal<HashMap<String, FSObj>>),
    Handle(RefOrVal<u128>),
    Null,
}

impl From<&FSObj> for String {
    fn from(a: &FSObj) -> Self {
        match a {
            FSObj::Int(n) => format!("{n}"),
            FSObj::String(s) => format!("\"{s}\""),
            FSObj::Boolean(b) => format!("{b:#}"),
            FSObj::Float(f) => format!("{f}"),
            FSObj::Double(d) => format!("{d}"),
            FSObj::Bytes(b) => format!("{b:?}"),
            FSObj::List(l) => {
                "[".to_string()
                    + &l.iter()
                        .map(String::from)
                        .collect::<Vec<String>>()
                        .join(", ")
                    + "]"
            }
            FSObj::Dist(d) => {
                "{".to_string()
                    + &d.iter()
                        .map(|(k, v)| format!("{}: {}", k, String::from(v)))
                        .collect::<Vec<String>>()
                        .join(", ")
                    + "}"
            }
            FSObj::Handle(h) => format!("Handle({h})"),
            FSObj::Null => String::new(),
        }
    }
}

impl FSObj {
    pub fn get_obj_mut(
        &mut self,
        path: String,
        allow_auto_digging: bool,
    ) -> Result<&mut FSObj, SyscallError> {
        let mut obj = self;
        for part in path.split('/') {
            match obj {
                FSObj::Dist(map) => {
                    if !map.contains_key(part) {
                        if allow_auto_digging {
                            map.insert(
                                part.to_string(),
                                FSObj::Dist(RefOrVal::Val(HashMap::new())),
                            );
                        } else {
                            return Err(SyscallError::NoSuchEntry);
                        }
                    }
                    obj = map.get_mut(part).unwrap();
                }
                _ => return Err(SyscallError::NoSuchEntry),
            }
        }

        Ok(obj)
    }
}
