use super::SyscallError;
use crate::obj_tree::FSObjRef;

#[derive(Debug, Clone, Default)]
pub enum SyscallData {
    #[default]
    None,
    Fail(SyscallError),
    FSRoot(FSObjRef),
    FSSuccess,
    FSGet(FSObjRef),
    Invoke {
        caller_pid: u64,
        obj: FSObjRef,
        arg: Option<FSObjRef>,
    },
    GetPid(u64),
}
