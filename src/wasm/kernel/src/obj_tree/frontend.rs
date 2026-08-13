use std::collections::HashMap;

use crate::{
    libs::split_filename,
    obj_tree::{fs_obj::Object, IntrinsicFSObj},
};

use super::{
    fs_obj::{FSObjRef, FileStat},
    fs_returns::FSReturns,
};

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
        self.resolve_(path)
            .map_err(|_| FSReturns::UnknownPath)?
            .list()
            .map_err(|_| FSReturns::UnknownError)
    }

    pub fn stat(&self, path: String) -> Result<FileStat, FSReturns> {
        let x = self
            .resolve_(path)
            .map_err(|_| FSReturns::UnknownPath)?
            .stat()
            .map_err(|_| FSReturns::UnknownError)?;

        Ok(x)
    }

    pub fn get(&self, path: String) -> Result<FSObjRef, FSReturns> {
        self.resolve_(path).map_err(|_| FSReturns::UnknownPath)
    }
    pub fn set(&self, path: String, obj: FSObjRef) -> Result<(), FSReturns> {
        let (parent, filename) =
            split_filename(path.clone()).ok_or(FSReturns::InvalidCommandFormat)?;

        self.resolve_(parent)
            .map_err(|_| FSReturns::UnknownPath)?
            .add_child(filename, obj)
            .map_err(|_| FSReturns::UnknownError)?;

        Ok(())
    }

    pub fn mkdir(&self, path: String, name: String) -> Result<(), FSReturns> {
        let parent = self.resolve_(path).map_err(|_| FSReturns::UnknownPath)?;

        let new_dir = IntrinsicFSObj::CompoundFSObj {
            parent: Some(parent.clone()),
            children: HashMap::default(),
        };
        parent
            .add_child(name, new_dir.into())
            .map_err(|_| FSReturns::UnknownError)?;

        Ok(())
    }
}
