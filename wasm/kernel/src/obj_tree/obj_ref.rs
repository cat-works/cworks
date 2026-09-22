use std::{
    cell::RefCell,
    fmt::{Debug, Display},
    ops::Deref,
    rc::{Rc, Weak},
};

use crate::SyscallError;

use super::Object;

#[derive(Clone)]
pub struct FSObjRef(Rc<RefCell<Box<Object>>>);

impl FSObjRef {
    #[must_use]
    pub fn new_compound(parent: Self) -> Self {
        let obj = Object::CompoundFSObj {
            parent: Some(Rc::downgrade(&parent.0)),
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
                    if let Some(parent) = parent.as_ref().and_then(Weak::upgrade) {
                        return Ok(Self(parent));
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
            *parent = Some(Rc::downgrade(&self.0));
        }

        Ok(())
    }

    pub fn remove_child(&self, name: &str) -> Result<Self, SyscallError> {
        let child = {
            let mut this = self.0.borrow_mut();
            match this.as_mut() {
                Object::CompoundFSObj { children, .. } => children.remove(name),
                _ => return Err(SyscallError::InvalidRequest),
            }
        };
        let Some(child) = child else {
            return Err(SyscallError::NoSuchEntry);
        };
        if let Object::CompoundFSObj { parent, .. } = child.0.borrow_mut().as_mut() {
            *parent = None;
        }
        Ok(child)
    }

    pub fn has_child(&self, name: &str) -> Result<bool, SyscallError> {
        match **self.0.borrow() {
            Object::CompoundFSObj { ref children, .. } => Ok(children.contains_key(name)),
            _ => Err(SyscallError::UnsupportedMethod),
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_child_sets_parent_and_remove_child_clears_it() {
        let root = FSObjRef::empty_compound();
        let child = FSObjRef::new_compound(root.clone());
        root.add_child("a", &child).unwrap();

        assert!(root.has_child("a").unwrap());
        assert_eq!(child.get_obj("..").unwrap().as_ptr(), root.as_ptr());

        let removed = root.remove_child("a").unwrap();
        assert!(!root.has_child("a").unwrap());
        assert_eq!(removed.as_ptr(), child.as_ptr());
        assert!(child.get_obj("..").is_err());
        assert!(root.remove_child("a").is_err());
    }
}
