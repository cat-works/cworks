mod kernel_process;
pub use kernel_process::KernelProcess;

#[derive(Debug, Clone)]
pub struct Handle {
    pub id: u128,
}

impl Handle {
    pub fn new(id: u128) -> Handle {
        Handle { id }
    }
}
#[derive(Debug, Clone)]
pub enum SyscallError {
    NoSuchEntry,   // no such entry(like file, socket or ipc)
    AlreadyExists, // entry already exists

    UnknownHandle, // not created handle or invalid handle
    // NotAllowedHandle, // handle is not allowed to use for the process
    NotImplemented,
}
impl std::fmt::Display for SyscallError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone, Debug)]
pub enum Syscall {
    Lock(String),
    IPC_Create(String),
    IPC_Connect(String),
}

#[derive(Debug, Clone)]
pub enum SyscallData {
    Handle(Result<Handle, SyscallError>),
    None,
}

impl Default for SyscallData {
    fn default() -> Self {
        SyscallData::None
    }
}

#[derive(Clone)]
pub enum PollResult<Ret> {
    Pending,
    Done(Ret),
    Syscall(Syscall),
}

impl<T> Default for PollResult<T> {
    fn default() -> Self {
        PollResult::Pending
    }
}

pub trait Process: Sync + Send {
    fn poll(&mut self, data: &SyscallData) -> PollResult<i64>;
}
