use std::collections::HashMap;

use crate::{process::SyscallError, Syscall};

#[derive(Debug, Clone)]
pub enum FSObj {
    Int(i128),
    String(String),
    Boolean(bool),
    Float(f32),
    Double(f64),
    Bytes(Vec<u8>),
    List(Vec<FSObj>),
    Dist(HashMap<String, FSObj>),
    Handle(u128),
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
                            map.insert(part.to_string(), FSObj::Dist(HashMap::new()));
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
