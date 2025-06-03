use std::collections::HashMap;

use crate::fs::{
    traits::{DaemonCommunicable, DaemonString},
    FSReturns,
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

    fn from_daemon_string(_: DaemonString) -> Result<Self, FSReturns>
    where
        Self: Sized,
    {
        Err(FSReturns::UnsupportedMethod)
    }
}

impl FSObj for CompoundFSObj {
    fn stat(&self) -> Result<super::fs_obj::FileStat, FSReturns> {
        Ok(FileStat {
            kind: super::fs_obj::FileKind::Directory,
        })
    }

    // directory-like methods
    fn list(&self) -> Result<Vec<String>, FSReturns> {
        // '.', '..' and all children
        let mut list = vec![".".to_string()];
        if self.parent.is_some() {
            list.push("..".to_string());
        }
        for child in self.children.keys() {
            list.push(child.clone());
        }
        Ok(list)
    }

    fn get_obj(&self, part: String) -> Result<FSObjRef, FSReturns> {
        if let Some(obj) = self.children.get(&part) {
            return Ok(obj.clone());
        }
        if part == ".." {
            if let Some(parent) = &self.parent {
                return parent.borrow().get_obj(part);
            }
        }
        Err(FSReturns::UnknownPath)
    }

    fn add_child(&mut self, name: String, obj: FSObjRef) -> Result<(), FSReturns> {
        self.children.insert(name, obj);

        Ok(())
    }
}
