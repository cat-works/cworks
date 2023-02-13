use std::{pin::Pin, sync::Arc};

use futures::Future;

mod dummy_future;
mod session;

use futures_task::{Context, Poll};
use kernel::{Process, SyscallError};
pub use session::Session;

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
    pub fn new(f: &impl Fn(Arc<Session>) -> F) -> Self {
        let session = Arc::new(Session::default());

        Self {
            f: f(session.clone()),
            session,
            ctx: Context::from_waker(futures_task::noop_waker_ref()),
        }
    }
}

impl<'a, F> Process for RustProcess<'a, F>
where
    F: Future<Output = Result<i64, SyscallError>> + Send + Sync,
{
    fn poll(&mut self, data: &kernel::SyscallData) -> kernel::PollResult<i64> {
        let f = unsafe { Pin::new_unchecked(&mut self.f) };

        self.session.set_syscall_data(data);
        let r = f.poll(&mut self.ctx);

        {
            let mut syscall = self.session.syscall.lock().unwrap();
            if syscall.is_some() {
                let r = kernel::PollResult::Syscall(syscall.as_ref().unwrap().clone());
                *syscall = None;
                return r;
            }
        }

        match r {
            Poll::Ready(Ok(v)) => kernel::PollResult::Done(v),
            Poll::Ready(Err(e)) => kernel::PollResult::Done(e.into()),
            Poll::Pending => kernel::PollResult::Pending,
        }
    }
}
