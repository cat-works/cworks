use super::SyscallError;
use crate::obj_tree::FSObjRef;

#[derive(Debug, Clone, Default)]
pub enum SyscallData {
    #[default]
    None,
    Fail(SyscallError),
    FSRoot(FSObjRef),
    FSSuccess,
    Invoke {
        caller_pid: u64,
        obj: FSObjRef,
        arg: FSObjRef,
    },
    GetPid(u64),
}
