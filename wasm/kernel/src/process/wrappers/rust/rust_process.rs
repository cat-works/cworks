use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use crate::{PollResult, Process, ProcessSession, SyscallData};

pub struct RustProcess<'a, F>
where
    F: Future<Output = ()>,
{
    f: F,
    session: ProcessSession,
    ctx: Context<'a>,
}

impl<F> RustProcess<'_, F>
where
    F: Future<Output = ()>,
{
    pub fn new<T>(f: &impl Fn(ProcessSession, T) -> F, arg: T) -> Self {
        let session = ProcessSession::default();

        Self {
            f: f(session.clone(), arg),
            session,
            ctx: Context::from_waker(futures_task::noop_waker_ref()),
        }
    }
}

impl<F> Process for RustProcess<'_, F>
where
    F: Future<Output = ()>,
{
    fn poll(&mut self, data: &SyscallData) -> PollResult {
        let f = unsafe { Pin::new_unchecked(&mut self.f) };

        self.session.pass_syscall_data(data);
        let r = f.poll(&mut self.ctx);

        match r {
            Poll::Ready(()) => return PollResult::Done,
            Poll::Pending => {}
        }

        self.session.take_syscall()
    }
}
