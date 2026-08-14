mod kernel_process;
mod poll_result;
mod syscall_data;
mod syscall_error;
mod wrappers;

pub use kernel_process::KernelProcess;
pub use kernel_process::ProcessStatus;
pub use poll_result::PollResult;
pub use syscall_data::SyscallData;
pub use syscall_error::SyscallError;
pub use wrappers::*;

use crate::handle::HandleRef;
use crate::obj_tree::FSObjRef;
use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Syscall {
    Sleep(f32),
    IpcCreate(String),
    IpcConnect(String),
    Send(HandleRef, String),
    WaitForProcess(u128),
    List(String),
    Stat(String),
    Get(String),
    Set(String, FSObjRef),
    Mkdir(String, String),
    Subscribe(String),
    Unsubscribe(String),
    Publish(String, Option<FSObjRef>),
}

pub trait Process {
    fn poll(&mut self, data: &SyscallData) -> PollResult<i64>;
}
