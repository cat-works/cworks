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

pub trait Process {
    fn poll(&mut self, data: &SyscallData) -> PollResult;
}
