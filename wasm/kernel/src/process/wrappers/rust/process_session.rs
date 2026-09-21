use std::{cell::RefCell, collections::HashMap, future::Future, ops::Deref, pin::Pin, rc::Rc};

use crate::{obj_tree::FSObjRef, PollResult, SyscallData};

type RawDataHandler = dyn Fn(FSObjRef) -> Pin<Box<dyn Future<Output = ()>>>;
pub type DataHandler = Rc<Box<RawDataHandler>>;

#[derive(Clone, Default)]
pub struct ProcessSession {
    pub(super) result: Rc<RefCell<PollResult>>,
    pub(super) response: Rc<RefCell<Option<SyscallData>>>,

    data_handlers: Rc<RefCell<HashMap<*const <FSObjRef as Deref>::Target, DataHandler>>>,
}

impl ProcessSession {
    pub(super) async fn do_syscall(&mut self, syscall: PollResult) -> SyscallData {
        self.response.borrow_mut().take();
        *self.result.borrow_mut() = syscall;
        loop {
            let res = std::future::poll_fn(|_| {
                self.response
                    .borrow_mut()
                    .take()
                    .map_or_else(|| std::task::Poll::Pending, std::task::Poll::Ready)
            })
            .await;
            self.result.take();

            if let SyscallData::Invoke {
                caller_pid: _,
                obj,
                arg,
            } = &res
            {
                let handler = self.data_handlers.borrow().get(&obj.as_ptr()).cloned();
                if let Some(handler) = handler {
                    handler(arg.clone()).await;
                }
            } else {
                return res;
            }
        }
    }

    pub(crate) fn register_data_handler(&mut self, obj: &FSObjRef, handler: DataHandler) {
        self.data_handlers
            .borrow_mut()
            .insert(obj.as_ptr(), handler);
    }
}
