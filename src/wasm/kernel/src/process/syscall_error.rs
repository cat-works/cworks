use serde::Serialize;

#[derive(PartialEq, Eq, Debug, Clone, Serialize)]
pub enum SyscallError {
    NoSuchEntry,   // no such entry
    AlreadyExists, // entry already exists

    NotImplemented,

    UnreachableEntry,

    ResourceIsBusy,

    UnsupportedMethod,
    InvalidRequest,
}

impl std::fmt::Display for SyscallError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
