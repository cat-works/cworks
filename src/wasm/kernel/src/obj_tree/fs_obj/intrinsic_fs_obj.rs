use std::{
    collections::HashMap,
    fmt::{Debug, Display},
    rc::Rc,
};

use serde::{Deserialize, Serialize};

use crate::obj_tree::FSObjRef;

#[derive(Serialize, Deserialize)]
pub enum IntrinsicFSObj {
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
}

impl Debug for IntrinsicFSObj {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IntrinsicFSObj::Int(x) => write!(f, "Int({x})"),
            IntrinsicFSObj::String(x) => write!(f, "String(\"{x}\")"),
            IntrinsicFSObj::Boolean(x) => write!(f, "Boolean({x:#})"),
            IntrinsicFSObj::Float(x) => write!(f, "Float({x})"),
            IntrinsicFSObj::Double(x) => write!(f, "Double({x})"),
            IntrinsicFSObj::Bytes(x) => write!(f, "Bytes({x:?})"),
            IntrinsicFSObj::Null => write!(f, "Null"),
            IntrinsicFSObj::CompoundFSObj { parent, children } => {
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
        }
    }
}

impl Display for IntrinsicFSObj {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IntrinsicFSObj::Int(x) => write!(f, "{x}"),
            IntrinsicFSObj::String(x) => write!(f, "\"{x}\""),
            IntrinsicFSObj::Boolean(x) => write!(f, "{x:#}"),
            IntrinsicFSObj::Float(x) => write!(f, "{x}"),
            IntrinsicFSObj::Double(x) => write!(f, "{x}"),
            IntrinsicFSObj::Bytes(x) => write!(f, "{x:?}"),
            IntrinsicFSObj::Null => write!(f, "Null"),
            IntrinsicFSObj::CompoundFSObj { parent, children } => {
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
        }
    }
}
