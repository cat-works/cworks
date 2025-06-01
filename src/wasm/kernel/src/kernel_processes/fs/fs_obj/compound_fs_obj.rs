use std::collections::HashMap;

use crate::{
    fs::{
        traits::{DaemonCommunicable, DaemonString},
        FSReturns,
    },
    SyscallError,
};

use super::{fs_obj::FileStat, FSObj, FSObjRef};

#[derive(Debug)]
pub struct CompoundFSObj {
    pub parent: Option<FSObjRef>,
    pub children: HashMap<String, FSObjRef>,
}
impl CompoundFSObj {
    pub fn new() -> Self {
        CompoundFSObj {
            parent: None,
            children: HashMap::new(),
        }
    }

    pub fn with_parent(parent: FSObjRef) -> Self {
        CompoundFSObj {
            parent: Some(parent),
            children: HashMap::new(),
        }
    }
}

impl DaemonCommunicable for CompoundFSObj {
    fn to_daemon_string(&self) -> Result<DaemonString, FSReturns> {
        Err(FSReturns::UnsupportedMethod)
    }

    fn from_daemon_string(s: DaemonString) -> Result<Self, FSReturns>
    where
        Self: Sized,
    {
        Err(FSReturns::UnsupportedMethod)
    }
}

impl FSObj for CompoundFSObj {
    fn stat(&self) -> Result<super::fs_obj::FileStat, SyscallError> {
        Ok(FileStat {
            kind: super::fs_obj::FileKind::Directory,
        })
    }

    // directory-like methods
    fn list(&self) -> Result<Vec<String>, SyscallError> {
        // '.', '..' and all children
        let mut list = vec![".".to_string()];
        if let Some(parent) = &self.parent {
            list.push("..".to_string());
        }
        for child in self.children.keys() {
            list.push(child.clone());
        }
        Ok(list)
    }

    fn get_obj(&self, part: String) -> Result<FSObjRef, SyscallError> {
        if let Some(obj) = self.children.get(&part) {
            return Ok(obj.clone());
        }
        if part == ".." {
            if let Some(parent) = &self.parent {
                return parent.borrow_mut().get_obj(part);
            }
        }
        Err(SyscallError::NoSuchEntry)
    }

    fn add_child(&mut self, name: String, obj: FSObjRef) -> Result<(), SyscallError> {
        self.children.insert(name, obj);

        Ok(())
    }
}
