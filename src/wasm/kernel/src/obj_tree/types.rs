use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Debug)]
pub enum FileKind {
    File,
    Directory,
    Function,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct FileStat {
    pub kind: FileKind,
}
