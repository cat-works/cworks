use std::{
    collections::HashMap,
    fmt::{Debug, Display},
};

use serde::{Deserialize, Serialize};

use crate::obj_tree::FSObjRef;

#[derive(Serialize, Deserialize)]
pub enum Object {
    Int(i128),
    String(String),
    Boolean(bool),
    Float(f32),
    Double(f64),
    Bytes(Vec<u8>),
    Null,
    CompoundFSObj {
        #[serde(skip_serializing)]
        parent: Option<FSObjRef>,
        children: HashMap<String, FSObjRef>,
    },
    #[serde(skip_serializing)]
    Func {
        callee_pid: Vec<u128>,
    },
}

impl Debug for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Object::Int(x) => write!(f, "Int({x})"),
            Object::String(x) => write!(f, "String(\"{x}\")"),
            Object::Boolean(x) => write!(f, "Boolean({x:#})"),
            Object::Float(x) => write!(f, "Float({x})"),
            Object::Double(x) => write!(f, "Double({x})"),
            Object::Bytes(x) => write!(f, "Bytes({x:?})"),
            Object::Null => write!(f, "Null"),
            Object::CompoundFSObj { parent, children } => {
                let parent_ptr = parent.as_ref().map(|x| x.as_ptr() as usize);
                let children = children
                    .iter()
                    .map(|(k, v)| format!("{}: {:?}", k, v))
                    .collect::<Vec<String>>()
                    .join(", ");
                write!(
                    f,
                    "CompoundFSObj {{ parent: {:?}, children: {} }}",
                    parent_ptr, children
                )
            }
            Object::Func { callee_pid } => f
                .debug_struct("Func")
                .field("callee_pid", callee_pid)
                .finish(),
        }
    }
}

impl Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Object::Int(x) => write!(f, "{x}"),
            Object::String(x) => write!(f, "\"{x}\""),
            Object::Boolean(x) => write!(f, "{x:#}"),
            Object::Float(x) => write!(f, "{x}"),
            Object::Double(x) => write!(f, "{x}"),
            Object::Bytes(x) => write!(f, "{x:?}"),
            Object::Null => write!(f, "Null"),
            Object::CompoundFSObj { parent, children } => {
                let parent_ptr = parent.as_ref().map(|x| x.as_ptr() as usize);
                let children = children
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, v))
                    .collect::<Vec<String>>()
                    .join(", ");
                write!(
                    f,
                    "CompoundFSObj {{ parent: {:?}, children: {} }}",
                    parent_ptr, children
                )
            }
            Object::Func { callee_pid } => {
                write!(f, "Func(owned by {:?})", callee_pid)
            }
        }
    }
}
