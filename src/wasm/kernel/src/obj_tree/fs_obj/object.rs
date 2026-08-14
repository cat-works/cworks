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
