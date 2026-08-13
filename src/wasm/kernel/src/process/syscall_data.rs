use super::SyscallError;
use crate::{
    obj_tree::{FSObjRef, FSReturns, FileStat},
    Handle,
};
use serde::Serialize;

#[derive(Debug, Clone, Serialize, Default)]
pub enum SyscallData {
    #[default]
    None,
    Fail(SyscallError),
    Handle(Handle),
    Connection {
        client: Handle,
        server: Handle,
    },
    ReceivingData {
        focus: Handle,
        data: String,
    },
    FSSuccess,
    FSGet(FSObjRef),
    FSError(FSReturns),
    FSList(Vec<String>),
    FSStat(FileStat),
}
