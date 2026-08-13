use serde::{Deserialize, Serialize};

use crate::obj_tree::FSReturns;
use std::fmt::Debug;

use super::FSObjRef;

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Debug)]
pub enum FileKind {
    File,
    Directory,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct FileStat {
    pub kind: FileKind,
}

impl From<Vec<u8>> for FileStat {
    fn from(bytes: Vec<u8>) -> Self {
        let kind = match bytes.first() {
            Some(0x66) => FileKind::File,
            Some(0x64) => FileKind::Directory,
            _ => FileKind::File, // Default to File if unknown
        };
        FileStat { kind }
    }
}

impl From<FileStat> for Vec<u8> {
    fn from(stat: FileStat) -> Self {
        let kind_byte = match stat.kind {
            FileKind::File => 0x66,
            FileKind::Directory => 0x64,
        };
        vec![kind_byte]
    }
}

pub trait Object: Debug {
    fn stat(&self) -> Result<FileStat, FSReturns>;

    // Directory-like methods
    fn list(&self) -> Result<Vec<String>, FSReturns> {
        Err(FSReturns::UnsupportedMethod)
    }
    fn get_obj(&self, _part: String) -> Result<FSObjRef, FSReturns> {
        Err(FSReturns::UnsupportedMethod)
    }
    fn add_child(&self, _name: String, _obj: FSObjRef) -> Result<(), FSReturns> {
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
            let next = current.get_obj(part.to_string())?;
            current = next;
        }

        Ok(current)
    }
}

impl<T: Object> Object for Box<T> {
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
