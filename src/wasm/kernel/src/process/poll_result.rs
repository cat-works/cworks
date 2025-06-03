use serde::{Deserialize, Serialize};

use crate::Syscall;

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub enum PollResult<Ret> {
    #[default]
    Pending,
    Done(Ret),
    Syscall(Syscall),
}
