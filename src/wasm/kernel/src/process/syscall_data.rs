use crate::Handle;

use super::SyscallError;

#[derive(Debug, Clone)]
pub enum SyscallData {
    Handle(Result<Handle, SyscallError>),
    Connection { client: Handle, server: Handle },
    ReceivingData { client: Handle, data: String },
    None,
}

impl Default for SyscallData {
    fn default() -> Self {
        SyscallData::None
    }
}
