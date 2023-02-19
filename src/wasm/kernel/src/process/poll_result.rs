use serde::{Deserialize, Serialize};

use crate::Syscall;

#[derive(Clone, Debug, Serialize, Deserialize)]
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
