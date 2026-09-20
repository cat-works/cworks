use std::{
    collections::HashMap,
    fmt::{Debug, Display},
};

use crate::obj_tree::FSObjRef;

pub enum Object {
    Int(i128),
    String(String),
    Boolean(bool),
    Float(f32),
    Double(f64),
    Bytes(Vec<u8>),
    Null,
    CompoundFSObj {
        parent: Option<FSObjRef>,
        children: HashMap<String, FSObjRef>,
    },
    Func {
        callee_pid: Vec<u64>,
    },
}

impl Debug for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Int(x) => write!(f, "Int({x})"),
            Self::String(x) => write!(f, "String(\"{x}\")"),
            Self::Boolean(x) => write!(f, "Boolean({x:#})"),
            Self::Float(x) => write!(f, "Float({x})"),
            Self::Double(x) => write!(f, "Double({x})"),
            Self::Bytes(x) => write!(f, "Bytes({x:?})"),
            Self::Null => write!(f, "Null"),
            Self::CompoundFSObj { parent, children } => {
                let parent_ptr = parent.as_ref().map(|x| x.as_ptr() as usize);
                let children = children
                    .iter()
                    .map(|(k, v)| format!("{k}: {v:?}"))
                    .collect::<Vec<String>>()
                    .join(", ");
                write!(
                    f,
                    "CompoundFSObj {{ parent: {parent_ptr:?}, children: {children} }}"
                )
            }
            Self::Func { callee_pid } => f
                .debug_struct("Func")
                .field("callee_pid", callee_pid)
                .finish(),
        }
    }
}

impl Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Int(x) => write!(f, "{x}"),
            Self::String(x) => write!(f, "\"{x}\""),
            Self::Boolean(x) => write!(f, "{x:#}"),
            Self::Float(x) => write!(f, "{x}"),
            Self::Double(x) => write!(f, "{x}"),
            Self::Bytes(x) => write!(f, "{x:?}"),
            Self::Null => write!(f, "Null"),
            Self::CompoundFSObj { parent, children } => {
                let parent_ptr = parent.as_ref().map(|x| x.as_ptr() as usize);
                let children = children
                    .iter()
                    .map(|(k, v)| format!("{k}: {v:?}"))
                    .collect::<Vec<String>>()
                    .join(", ");
                write!(
                    f,
                    "CompoundFSObj {{ parent: {parent_ptr:?}, children: {children} }}"
                )
            }
            Self::Func { callee_pid } => {
                write!(f, "Func(owned by {callee_pid:?})")
            }
        }
    }
}
