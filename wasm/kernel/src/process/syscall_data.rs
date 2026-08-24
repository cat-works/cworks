use super::SyscallError;
use crate::obj_tree::{FSObjRef, FileStat};
use serde::Serialize;

#[derive(Debug, Clone, Serialize, Default)]
pub enum SyscallData {
    #[default]
    None,
    Fail(SyscallError),
    FSSuccess,
    FSGet(FSObjRef),
    FSList(Vec<String>),
    FSStat(FileStat),
    Invoke {
        caller_pid: u128,
        path: String,
        arg: Option<FSObjRef>,
    },
    GetPid(u128),
}
