use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use log::Level::Trace;

use crate::{PollResult, Process, SyscallData};

use super::process::RustProcessCore;

pub struct RustProcess<'a, F>
where
    F: Future<Output = i64>,
{
    f: F,
    session: RustProcessCore,
    ctx: Context<'a>,
}

impl<F> RustProcess<'_, F>
where
    F: Future<Output = i64>,
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
    F: Future<Output = i64>,
{
    fn poll(&mut self, data: &SyscallData) -> PollResult {
        let f = unsafe { Pin::new_unchecked(&mut self.f) };

        self.session.set_syscall_data(data);
        let r = f.poll(&mut self.ctx);

        log::trace!("RustProcess::poll: data: {data:?}, result: {r:?}");

        match r {
            Poll::Ready(v) => return PollResult::Done(v),
            Poll::Pending => {}
        }

        let res = self.session.result.borrow().clone();
        self.session.result.replace(PollResult::Pending);

        res
    }
}
