mod kernel_process;
mod poll_result;
mod syscall_data;
mod syscall_error;

use crate::Handle;
pub use kernel_process::KernelProcess;
pub use kernel_process::ProcessStatus;
pub use poll_result::PollResult;
pub use syscall_data::SyscallData;
pub use syscall_error::SyscallError;

#[derive(Clone, Debug)]
pub enum Syscall {
    IPC_Create(String),
    IPC_Connect(String),
    Send(Option<Handle>, Handle, String),
}

pub trait Process: Sync + Send {
    fn poll(&mut self, data: &SyscallData) -> PollResult<i64>;
}
