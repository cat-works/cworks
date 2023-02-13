use crate::Syscall;

#[derive(Clone, Debug)]
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
