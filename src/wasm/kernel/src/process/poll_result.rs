use serde::Deserialize;

use crate::Syscall;

#[derive(Clone, Debug, Deserialize)]
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
