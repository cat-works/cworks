use super::SyscallError;
use crate::Handle;
use serde::Serialize;

#[derive(PartialEq, Debug, Clone, Serialize, Default)]
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
    FSResult(String),
}
