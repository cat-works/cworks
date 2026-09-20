use std::{
    cell::RefCell,
    fmt::{Debug, Display},
    ops::Deref,
    rc::Rc,
};

use crate::SyscallError;

use super::Object;

#[derive(Clone)]
pub struct FSObjRef(Rc<RefCell<Box<Object>>>);

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
    // directory-like methods
    pub fn list(&self) -> Result<Vec<String>, SyscallError> {
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

            _ => Err(SyscallError::InvalidRequest),
        }
    }

    pub fn get_obj(&self, part: &str) -> Result<Self, SyscallError> {
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

                Err(SyscallError::NoSuchEntry)
            }

            _ => Err(SyscallError::UnsupportedMethod),
        }
    }

    pub fn add_child(&self, name: &str, obj: &Self) -> Result<(), SyscallError> {
        match **self.0.clone().borrow_mut() {
            Object::CompoundFSObj {
                ref mut children, ..
            } => {
                children.insert(name.to_string(), obj.clone());
            }

            _ => return Err(SyscallError::InvalidRequest),
        }
        if let Object::CompoundFSObj { ref mut parent, .. } = **obj.0.borrow_mut() {
            *parent = Some(self.clone());
        }

        Ok(())
    }

    pub fn follow(&self, path: &str) -> Result<Self, SyscallError> {
        let parts = path.split('/').filter(|x| !x.is_empty());

        let mut current: Self = self.clone();

        for part in parts {
            let next = current.get_obj(part)?;
            current = next;
        }

        Ok(current)
    }
}
