use std::collections::HashMap;

use crate::libs::split_filename;

use super::{FSObjRef, FSReturns, FileStat, Object};

pub struct FSFrontend {
    pub root: FSObjRef,
}

impl FSFrontend {
    pub fn new(root: FSObjRef) -> Self {
        Self { root }
    }

    fn resolve_(&self, path: String) -> Result<FSObjRef, FSReturns> {
        if path == "/" || path.is_empty() {
            // Root path
            Ok(self.root.clone())
        } else if path.starts_with("/") {
            // Absolute path
            self.root.follow(path)
        } else {
            todo!("Relative paths are not supported yet")
        }
    }

    pub fn list(&self, path: String) -> Result<Vec<String>, FSReturns> {
        self.resolve_(path)?.list()
    }

    pub fn stat(&self, path: String) -> Result<FileStat, FSReturns> {
        self.resolve_(path)?.stat()
    }

    pub fn get(&self, path: String) -> Result<FSObjRef, FSReturns> {
        self.resolve_(path)
    }
    pub fn set(&self, path: String, obj: FSObjRef) -> Result<(), FSReturns> {
        let (parent, filename) =
            split_filename(path.clone()).ok_or(FSReturns::InvalidCommandFormat)?;

        self.resolve_(parent)?.add_child(filename, obj)?;

        Ok(())
    }

    pub fn mkdir(&self, path: String, name: String) -> Result<(), FSReturns> {
        let parent = self.resolve_(path)?;

        if let Ok(x) = parent.get_obj(name.clone()) {
            if let Object::CompoundFSObj { .. } = **x.borrow() {
                return Ok(());
            } else {
                return Err(FSReturns::InvalidCommandFormat);
            }
        }

        let new_dir = Object::CompoundFSObj {
            parent: Some(parent.clone()),
            children: HashMap::default(),
        };
        parent.add_child(name, new_dir.into())?;

        Ok(())
    }
}
