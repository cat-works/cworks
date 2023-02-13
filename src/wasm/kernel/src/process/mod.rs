mod kernel_process;
mod poll_result;
mod syscall_data;
mod syscall_error;

use std::sync::Arc;

use crate::Handle;
pub use kernel_process::KernelProcess;
pub use kernel_process::ProcessStatus;
pub use poll_result::PollResult;
pub use syscall_data::SyscallData;
pub use syscall_error::SyscallError;

#[derive(Clone, Debug)]
pub enum Syscall {
    Sleep(f32),
    IpcCreate(String),
    IpcConnect(String),
    Send(Arc<Handle>, String),
}

pub trait Process: Sync + Send {
    fn poll(&mut self, data: &SyscallData) -> PollResult<i64>;
}
