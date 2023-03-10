use serde::Serialize;

use crate::Handle;

use super::SyscallError;

#[derive(Debug, Clone, Serialize)]
pub enum SyscallData {
    Fail(SyscallError),
    Handle(Handle),
    Connection { client: Handle, server: Handle },
    ReceivingData { focus: Handle, data: String },
    None,
}

impl Default for SyscallData {
    fn default() -> Self {
        SyscallData::None
    }
}
