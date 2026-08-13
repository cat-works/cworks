use std::{cell::RefCell, fmt::Debug, rc::Rc};

use serde::Serialize;

use crate::obj_tree::{
    fs_obj::{object::FileKind, Object},
    FSReturns, FileStat, IntrinsicFSObj,
};

#[derive(Clone)]
pub struct FSObjRef(Rc<RefCell<Box<IntrinsicFSObj>>>);

impl Serialize for FSObjRef {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let obj = self.0.borrow();
        obj.serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for FSObjRef {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let obj = IntrinsicFSObj::deserialize(deserializer)?;
        Ok(FSObjRef(Rc::new(RefCell::new(Box::new(obj)))))
    }
}

impl FSObjRef {
    pub fn new_compound(parent: FSObjRef) -> Self {
        let obj = IntrinsicFSObj::CompoundFSObj {
            parent: Some(parent),
            children: std::collections::HashMap::new(),
        };
        FSObjRef(Rc::new(RefCell::new(Box::new(obj))))
    }
    pub fn empty_compound() -> Self {
        let obj = IntrinsicFSObj::CompoundFSObj {
            parent: None,
            children: std::collections::HashMap::new(),
        };
        FSObjRef(Rc::new(RefCell::new(Box::new(obj))))
    }
}

impl From<IntrinsicFSObj> for FSObjRef {
    fn from(obj: IntrinsicFSObj) -> Self {
        FSObjRef(Rc::new(RefCell::new(Box::new(obj))))
    }
}

impl Debug for FSObjRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

impl Object for FSObjRef {
    fn stat(&self) -> Result<FileStat, FSReturns> {
        match **self.0.borrow() {
            IntrinsicFSObj::CompoundFSObj { .. } => Ok(FileStat {
                kind: FileKind::Directory,
            }),
            _ => Ok(FileStat {
                kind: FileKind::File,
            }),
        }
    }

    // directory-like methods
    fn list(&self) -> Result<Vec<String>, FSReturns> {
        match **self.0.borrow() {
            IntrinsicFSObj::CompoundFSObj {
                ref parent,
                ref children,
            } => {
                let mut list = vec![".".to_string()];
                if parent.is_some() {
                    list.push("..".to_string());
                }
                for child in children.keys() {
                    list.push(child.clone());
                }
                Ok(list)
            }

            _ => Err(FSReturns::UnsupportedMethod),
        }
    }

    fn get_obj(&self, part: String) -> Result<FSObjRef, FSReturns> {
        match **self.0.borrow() {
            IntrinsicFSObj::CompoundFSObj {
                ref parent,
                ref children,
            } => {
                if let Some(obj) = children.get(&part) {
                    return Ok(obj.clone());
                }
                if part == ".." {
                    if let Some(parent) = &parent {
                        return parent.get_obj(part);
                    }
                }
                Err(FSReturns::UnknownPath)
            }

            _ => Err(FSReturns::UnsupportedMethod),
        }
    }

    fn add_child(&self, name: String, obj: FSObjRef) -> Result<(), FSReturns> {
        match **self.0.borrow_mut() {
            IntrinsicFSObj::CompoundFSObj {
                ref mut children, ..
            } => {
                children.insert(name, obj);
                Ok(())
            }

            _ => Err(FSReturns::UnsupportedMethod),
        }
    }
}
