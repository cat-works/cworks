use std::{collections::HashMap, sync::Arc};

use futures::lock::Mutex;

use crate::{process::SyscallError, Handle};

use super::RefOrVal;

pub trait DynamicFSObj: std::fmt::Debug + Send + Sync {
    fn hash(&self) -> u64;
    fn get_obj(&self, path: String) -> Result<&FSObj, SyscallError>;
    fn set_obj(&mut self, path: String, obj: FSObj) -> Result<(), SyscallError>;
}

#[derive(Debug)]
pub enum FSObj {
    Int(RefOrVal<i128>),
    String(RefOrVal<String>),
    Boolean(RefOrVal<bool>),
    Float(RefOrVal<f32>),
    Double(RefOrVal<f64>),
    Bytes(RefOrVal<Vec<u8>>),
    List(RefOrVal<Vec<FSObj>>),
    Dist(RefOrVal<HashMap<String, FSObj>>),
    Handle(RefOrVal<Handle>),
    Dynamic(Arc<Mutex<dyn DynamicFSObj>>),
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
            FSObj::Dynamic(d) => format!("{d:?}"),
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
