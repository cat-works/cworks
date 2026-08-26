use super::SyscallError;
use crate::obj_tree::{FSObjRef, FileStat};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum SyscallData {
    #[default]
    None,
    Fail(SyscallError),
    FSSuccess,
    FSGet(FSObjRef),
    FSList(Vec<String>),
    FSStat(FileStat),
    Invoke {
        caller_pid: u64,
        path: String,
        arg: Option<FSObjRef>,
    },
    GetPid(u64),
}
