use super::{Process, SyscallData};

#[derive(Debug, Clone)]
pub enum ProcessStatus {
    Running,
    Sleeping(f32),
}

pub struct KernelProcess {
    pub parent_pid: u128,
    pub process: Box<dyn Process>,
    pub system_call_returns: SyscallData,
    pub status: ProcessStatus,
}

impl From<Box<dyn Process>> for KernelProcess {
    fn from(p: Box<dyn Process>) -> Self {
        KernelProcess {
            parent_pid: 0,
            process: p,
            system_call_returns: SyscallData::None,
            status: ProcessStatus::Running,
        }
    }
}
