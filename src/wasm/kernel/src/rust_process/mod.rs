use std::{pin::Pin, sync::Arc};

use futures::Future;

mod dummy_future;
mod session;

use futures_task::{Context, Poll};

pub use session::Session;

use crate::{PollResult, Process, SyscallData, SyscallError};

pub struct RustProcess<'a, F>
where
    F: Future<Output = Result<i64, SyscallError>> + Send + Sync,
{
    f: F,
    session: Arc<Session>,
    ctx: Context<'a>,
}

impl<'a, F> RustProcess<'a, F>
where
    F: Future<Output = Result<i64, SyscallError>> + Send + Sync,
{
    pub fn new<T>(f: &impl Fn(Arc<Session>, T) -> F, data: T) -> Self {
        let session = Arc::new(Session::default());

        Self {
            f: f(session.clone(), data),
            session,
            ctx: Context::from_waker(futures_task::noop_waker_ref()),
        }
    }
}

impl<'a, F> Process for RustProcess<'a, F>
where
    F: Future<Output = Result<i64, SyscallError>> + Send + Sync,
{
    fn poll(&mut self, data: &SyscallData) -> PollResult<i64> {
        let f = unsafe { Pin::new_unchecked(&mut self.f) };

        self.session.set_syscall_data(data);
        let r = f.poll(&mut self.ctx);

        {
            let mut syscall = self.session.syscall.lock().unwrap();
            if syscall.is_some() {
                let r = PollResult::Syscall(syscall.as_ref().unwrap().clone());
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
