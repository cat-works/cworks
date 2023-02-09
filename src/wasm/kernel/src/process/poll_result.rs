use crate::Syscall;

#[derive(Clone)]
pub enum PollResult<Ret> {
    Pending,
    Done(Ret),
    Syscall(Syscall),
    Sleep(f32),
}

impl<T> Default for PollResult<T> {
    fn default() -> Self {
        PollResult::Pending
    }
}
