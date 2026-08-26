use std::{cell::RefCell, future::Future, rc::Rc};

use crate::{PollResult, SyscallData};

#[derive(Clone, Default)]
pub struct ProcessSession {
    pub(super) result: Rc<RefCell<PollResult>>,
    pub(super) response: Rc<RefCell<Option<SyscallData>>>,
}

impl ProcessSession {
    pub(super) fn do_syscall(
        &mut self,
        syscall: PollResult,
    ) -> impl Future<Output = SyscallData> + '_ {
        self.response.borrow_mut().take();
        *self.result.borrow_mut() = syscall;
        std::future::poll_fn(|_| {
            self.response
                .borrow_mut()
                .take()
                .map_or_else(|| std::task::Poll::Pending, std::task::Poll::Ready)
        })
    }
}
