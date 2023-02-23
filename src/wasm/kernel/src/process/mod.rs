mod kernel_process;
mod poll_result;
mod syscall_data;
mod syscall_error;
mod wrappers;

pub use kernel_process::*;
pub use poll_result::PollResult;
pub use syscall_data::SyscallData;
pub use syscall_error::SyscallError;
pub use wrappers::*;

use crate::Handle;
use serde::Deserialize;
use serde::Serialize;

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
