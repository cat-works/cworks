use std::sync::Arc;

use crate::Handle;

use super::SyscallError;

#[derive(Debug, Clone)]
pub enum SyscallData {
    Handle(Result<Arc<Handle>, SyscallError>),
    Connection {
        client: Arc<Handle>,
        server: Arc<Handle>,
    },
    ReceivingData {
        client: Arc<Handle>,
        data: String,
    },
    None,
}

impl Default for SyscallData {
    fn default() -> Self {
        SyscallData::None
    }
}
