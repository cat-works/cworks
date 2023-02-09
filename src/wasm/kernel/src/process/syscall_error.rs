#[derive(Debug, Clone)]
pub enum SyscallError {
    NoSuchEntry,   // no such entry(like file, socket or ipc)
    AlreadyExists, // entry already exists

    UnknownHandle, // not created handle or invalid handle
    // NotAllowedHandle, // handle is not allowed to use for the process
    NotImplemented,

    UnreachableEntry,
}
impl std::fmt::Display for SyscallError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
