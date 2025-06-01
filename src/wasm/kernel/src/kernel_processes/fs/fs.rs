use crate::{kernel_processes::path::split_filename, SyscallError};

use super::{
    fs_obj::{FSObj, FSObjRef, FileStat, IntrinsicFSObj},
    fs_returns::FSReturns,
};

pub struct FS {
    pub root: FSObjRef,
}

impl FS {
    pub fn new(root: FSObjRef) -> Self {
        Self { root }
    }

    fn resolve_(&self, path: String) -> Result<FSObjRef, SyscallError> {
        if path == "/" {
            // Root path
            return Ok(self.root.clone());
        } else if path.starts_with("/") {
            // Absolute path
            self.root.borrow_mut().follow(path)
        } else {
            todo!("Relative paths are not supported yet")
        }
    }

    pub fn exists(&self, path: String) -> bool {
        self.root.borrow().get_obj(path).is_ok()
    }

    pub fn list(&self, path: String) -> Result<Vec<String>, FSReturns> {
        self.resolve_(path)
            .map_err(|_| FSReturns::UnknownPath)?
            .borrow_mut()
            .list()
            .map_err(|_| FSReturns::UnknownError)
    }

    pub fn stat(&self, path: String) -> Result<FileStat, FSReturns> {
        self.resolve_(path)
            .map_err(|_| FSReturns::UnknownPath)?
            .borrow_mut()
            .stat()
            .map_err(|_| FSReturns::UnknownError)
    }

    pub fn get(&self, path: String) -> Result<FSObjRef, FSReturns> {
        let (parent, filename) = split_filename(path).ok_or(FSReturns::InvalidCommandFormat)?;

        self.resolve_(parent)
            .map_err(|_| FSReturns::UnknownPath)?
            .borrow_mut()
            .get_obj(filename)
            .map_err(|_| FSReturns::UnknownPath)
    }
    pub fn set(&self, path: String, obj: FSObjRef) -> Result<(), FSReturns> {
        let (parent, filename) = split_filename(path).ok_or(FSReturns::InvalidCommandFormat)?;

        self.resolve_(parent)
            .map_err(|_| FSReturns::UnknownPath)?
            .borrow_mut()
            .add_child(filename, obj)
            .map_err(|_| FSReturns::UnknownError)
    }
}
