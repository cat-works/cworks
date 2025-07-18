use crate::fs::{
    traits::{DaemonCommunicable, DaemonString},
    FSReturns,
};
use std::fmt::Debug;

use super::FSObjRef;

#[derive(Clone)]
pub enum FileKind {
    File,
    Directory,
}

#[derive(Clone)]
pub struct FileStat {
    pub kind: FileKind,
}

impl DaemonCommunicable for FileStat {
    fn to_daemon_string(&self) -> Result<DaemonString, FSReturns> {
        match self.kind {
            FileKind::File => Ok("f".into()),
            FileKind::Directory => Ok("d".into()),
        }
    }

    fn from_daemon_string(s: DaemonString) -> Result<Self, FSReturns>
    where
        Self: Sized,
    {
        match s.as_str() {
            "f" => Ok(FileStat {
                kind: FileKind::File,
            }),
            "d" => Ok(FileStat {
                kind: FileKind::Directory,
            }),
            _ => Err(FSReturns::UnsupportedMethod),
        }
    }
}

pub trait Object: Debug + DaemonCommunicable {
    fn stat(&self) -> Result<FileStat, FSReturns>;

    // Directory-like methods
    fn list(&self) -> Result<Vec<String>, FSReturns> {
        Err(FSReturns::UnsupportedMethod)
    }
    fn get_obj(&self, _part: String) -> Result<FSObjRef, FSReturns> {
        Err(FSReturns::UnsupportedMethod)
    }
    fn add_child(&mut self, _name: String, _obj: FSObjRef) -> Result<(), FSReturns> {
        Err(FSReturns::UnsupportedMethod)
    }

    // misc
    fn follow(&self, path: String) -> Result<FSObjRef, FSReturns> {
        let parts = path
            .split('/')
            .filter(|x| !x.is_empty())
            .collect::<Vec<_>>();

        let mut parts_iter = parts.iter();

        let first = match parts_iter.next() {
            Some(p) => p,
            None => return Err(FSReturns::UnsupportedMethod),
        };
        let mut current: FSObjRef = self.get_obj(first.to_string())?;

        for part in parts_iter {
            // Clone the reference to avoid holding the borrow across assignment
            let next = {
                let borrowed = current.borrow();
                borrowed.get_obj(part.to_string())?
            };
            current = next;
        }

        Ok(current)
    }
}

impl<T: Object + DaemonCommunicable> Object for Box<T> {
    fn get_obj(&self, part: String) -> Result<FSObjRef, FSReturns> {
        self.as_ref().get_obj(part)
    }

    fn stat(&self) -> Result<FileStat, FSReturns> {
        self.as_ref().stat()
    }

    fn follow(&self, path: String) -> Result<FSObjRef, FSReturns> {
        self.as_ref().follow(path)
    }
}
