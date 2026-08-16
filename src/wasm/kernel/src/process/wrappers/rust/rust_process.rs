use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use crate::{PollResult, Process, SyscallData};

use super::process::RustProcessCore;

pub struct RustProcess<'a, F>
where
    F: Future<Output = ()>,
{
    f: F,
    session: RustProcessCore,
    ctx: Context<'a>,
}

impl<F> RustProcess<'_, F>
where
    F: Future<Output = ()>,
{
    pub fn new<T>(f: &impl Fn(RustProcessCore, T) -> F, arg: T) -> Self {
        let session = RustProcessCore::default();

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

        self.session.set_syscall_data(data);
        let r = f.poll(&mut self.ctx);

        match r {
            Poll::Ready(()) => return PollResult::Done,
            Poll::Pending => {}
        }

        let res = self.session.result.borrow().clone();
        self.session.result.replace(PollResult::Pending);

        res
    }
}
