use std::{
    cell::RefCell,
    fmt::{Debug, Display},
    ops::Deref,
    rc::Rc,
};

use serde::Serialize;

use super::{FSReturns, FileKind, FileStat, Object};

#[derive(Clone)]
pub struct FSObjRef(Rc<RefCell<Box<Object>>>);

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
        let obj = Object::deserialize(deserializer)?;
        Ok(Self(Rc::new(RefCell::new(Box::new(obj)))))
    }
}

impl FSObjRef {
    #[must_use]
    pub fn new_compound(parent: Self) -> Self {
        let obj = Object::CompoundFSObj {
            parent: Some(parent),
            children: std::collections::HashMap::new(),
        };
        Self(Rc::new(RefCell::new(Box::new(obj))))
    }

    #[must_use]
    pub fn empty_compound() -> Self {
        let obj = Object::CompoundFSObj {
            parent: None,
            children: std::collections::HashMap::new(),
        };
        Self(Rc::new(RefCell::new(Box::new(obj))))
    }

    #[must_use]
    pub fn as_ptr(&self) -> *const RefCell<Box<Object>> {
        Rc::as_ptr(&self.0)
    }
}

impl From<Object> for FSObjRef {
    fn from(obj: Object) -> Self {
        Self(Rc::new(RefCell::new(Box::new(obj))))
    }
}

impl Deref for FSObjRef {
    type Target = RefCell<Box<Object>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Debug for FSObjRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self.0.borrow().as_ref(), f)
    }
}

impl Display for FSObjRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self.0.borrow().as_ref(), f)
    }
}

impl FSObjRef {
    #[must_use]
    pub fn stat(&self) -> FileStat {
        match **self.0.borrow() {
            Object::CompoundFSObj { .. } => FileStat {
                kind: FileKind::Directory,
            },
            Object::Func { .. } => FileStat {
                kind: FileKind::Function,
            },
            _ => FileStat {
                kind: FileKind::File,
            },
        }
    }

    // directory-like methods
    pub fn list(&self) -> Result<Vec<String>, FSReturns> {
        match **self.0.borrow() {
            Object::CompoundFSObj {
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

    pub fn get_obj(&self, part: &str) -> Result<Self, FSReturns> {
        match **self.0.borrow() {
            Object::CompoundFSObj {
                ref parent,
                ref children,
            } => {
                if let Some(obj) = children.get(part) {
                    return Ok(obj.clone());
                }
                if part == "." {
                    return Ok(self.clone());
                }
                if part == ".." {
                    if let Some(parent) = &parent {
                        return Ok(parent.clone());
                    }
                }

                Err(FSReturns::UnknownPath)
            }

            _ => Err(FSReturns::UnsupportedMethod),
        }
    }

    pub fn add_child(&self, name: &str, obj: &Self) -> Result<(), FSReturns> {
        match **self.0.clone().borrow_mut() {
            Object::CompoundFSObj {
                ref mut children, ..
            } => {
                children.insert(name.to_string(), obj.clone());
            }

            _ => return Err(FSReturns::UnsupportedMethod),
        }
        if let Object::CompoundFSObj { ref mut parent, .. } = **obj.0.borrow_mut() {
            *parent = Some(self.clone());
        }

        Ok(())
    }

    pub fn follow(&self, path: &str) -> Result<Self, FSReturns> {
        let parts = path.split('/').filter(|x| !x.is_empty());

        let mut current: Self = self.clone();

        for part in parts {
            let next = current.get_obj(part)?;
            current = next;
        }

        Ok(current)
    }
}
