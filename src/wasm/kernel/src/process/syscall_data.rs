use super::SyscallError;
use crate::{
    handle::HandleRef,
    obj_tree::{FSObjRef, FSReturns, FileStat},
};
use serde::Serialize;

#[derive(Debug, Clone, Serialize, Default)]
pub enum SyscallData {
    #[default]
    None,
    Fail(SyscallError),
    Handle(HandleRef),
    Connection {
        client: HandleRef,
        server: HandleRef,
    },
    ReceivingData {
        focus: HandleRef,
        data: String,
    },
    FSSuccess,
    FSGet(FSObjRef),
    FSError(FSReturns),
    FSList(Vec<String>),
    FSStat(FileStat),
}
