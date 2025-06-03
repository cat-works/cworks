use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use crate::{PollResult, Process, SyscallData, SyscallError};

use super::process::RustProcessCore;

pub struct RustProcess<'a, F>
where
    F: Future<Output = Result<i64, SyscallError>>,
{
    f: F,
    session: RustProcessCore,
    ctx: Context<'a>,
}

impl<'a, F> RustProcess<'a, F>
where
    F: Future<Output = Result<i64, SyscallError>>,
{
    pub fn new<T>(f: &impl Fn(RustProcessCore, T) -> F, arg: T) -> Self {
        let session = RustProcessCore::new();

        Self {
            f: f(session.clone(), arg),
            session,
            ctx: Context::from_waker(futures_task::noop_waker_ref()),
        }
    }
}

impl<'a, F> Process for RustProcess<'a, F>
where
    F: Future<Output = Result<i64, SyscallError>>,
{
    fn poll(&mut self, data: &SyscallData) -> PollResult<i64> {
        let f = unsafe { Pin::new_unchecked(&mut self.f) };

        self.session.set_syscall_data(data);
        let r = f.poll(&mut self.ctx);

        {
            let mut syscall = self.session.syscall.borrow_mut();
            if syscall.is_some() {
                let r = PollResult::Syscall(syscall.take().unwrap());
                *syscall = None;
                return r;
            }
        }

        match r {
            Poll::Ready(Ok(v)) => PollResult::Done(v),
            Poll::Ready(Err(e)) => PollResult::Done(e.into()),
            Poll::Pending => PollResult::Pending,
        }
    }
}
