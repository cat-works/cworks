mod kernel_process;
mod poll_result;
mod syscall_data;
mod syscall_error;

use crate::Handle;
pub use kernel_process::KernelProcess;
pub use kernel_process::ProcessStatus;
pub use poll_result::PollResult;
use serde::Deserialize;
use serde::Serialize;
pub use syscall_data::SyscallData;
pub use syscall_error::SyscallError;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Syscall {
    Sleep(f32),
    IpcCreate(String),
    IpcConnect(String),
    Send(Handle, String),
}

pub trait Process {
    fn poll(&mut self, data: &SyscallData) -> PollResult<i64>;
}
