use std::{cell::RefCell, collections::HashMap, future::Future, ops::Deref, pin::Pin, rc::Rc};

use crate::{obj_tree::FSObjRef, PollResult, SyscallData, SyscallError};

type RawDataHandler = dyn Fn(FSObjRef) -> Pin<Box<dyn Future<Output = ()>>>;
pub type DataHandler = Rc<Box<RawDataHandler>>;

#[derive(Default)]
pub struct ProcessSessionData {
    result: PollResult,
    response: Option<SyscallData>,

    data_handlers: HashMap<*const <FSObjRef as Deref>::Target, DataHandler>,
}

#[derive(Default, Clone)]
pub struct ProcessSession(Rc<RefCell<ProcessSessionData>>);

impl Deref for ProcessSession {
    type Target = RefCell<ProcessSessionData>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ProcessSession {
    pub(super) async fn do_syscall(&mut self, syscall: PollResult) -> SyscallData {
        self.borrow_mut().response = None;
        self.borrow_mut().result = syscall;
        loop {
            let res = std::future::poll_fn(|_| {
                self.borrow_mut()
                    .response
                    .take()
                    .map_or_else(|| std::task::Poll::Pending, std::task::Poll::Ready)
            })
            .await;
            self.borrow_mut().result = PollResult::Pending;

            if let SyscallData::Invoke {
                caller_pid: _,
                obj,
                arg,
            } = &res
            {
                let handler = self.borrow_mut().data_handlers.get(&obj.as_ptr()).cloned();
                if let Some(handler) = handler {
                    handler(arg.clone()).await;
                }
            } else {
                return res;
            }
        }
    }
    pub fn pass_syscall_data(&mut self, data: &SyscallData) {
        self.borrow_mut().response = Some(data.clone());
    }
    pub fn take_syscall(&mut self) -> PollResult {
        self.borrow_mut().result.clone()
    }

    //* System calls

    pub async fn sleep(&mut self, seconds: f32) {
        self.do_syscall(PollResult::Sleep(seconds)).await;
    }

    pub async fn subscribe(
        &mut self,
        obj: FSObjRef,
        handler: DataHandler,
    ) -> Result<(), SyscallError> {
        self.borrow_mut()
            .data_handlers
            .insert(obj.as_ptr(), handler);
        match self.do_syscall(PollResult::Subscribe(obj)).await {
            SyscallData::Fail(ref e) => Err(e.clone()),
            _ => Ok(()),
        }
    }

    pub async fn unsubscribe(&mut self, obj: FSObjRef) -> Result<(), SyscallError> {
        match self.do_syscall(PollResult::Unsubscribe(obj)).await {
            SyscallData::Fail(ref e) => Err(e.clone()),
            _ => Ok(()),
        }
    }

    pub async fn publish(&mut self, obj: FSObjRef, data: FSObjRef) -> Result<(), SyscallError> {
        match self.do_syscall(PollResult::Publish(obj, data)).await {
            SyscallData::Fail(ref e) => Err(e.clone()),
            _ => Ok(()),
        }
    }

    pub async fn fs_root(&mut self) -> Result<FSObjRef, SyscallError> {
        match self.do_syscall(PollResult::Root).await {
            SyscallData::Fail(ref e) => Err(e.clone()),
            SyscallData::FSRoot(e) => Ok(e),
            _ => Err(SyscallError::UnreachableEntry),
        }
    }

    pub async fn wait_for_event(&mut self) -> Result<(), SyscallError> {
        self.do_syscall(PollResult::WaitForEvent).await;
        Ok(())
    }

    pub async fn wait_for_process(&mut self, pid: u64) -> Result<(), SyscallError> {
        match self.do_syscall(PollResult::WaitForProcess(pid)).await {
            SyscallData::Fail(ref e) => Err(e.clone()),
            _ => Ok(()),
        }
    }

    pub async fn get_pid(&mut self) -> Result<u64, SyscallError> {
        match self.do_syscall(PollResult::GetPid).await {
            SyscallData::GetPid(pid) => Ok(pid),
            SyscallData::Fail(ref e) => Err(e.clone()),
            _ => Err(SyscallError::NotImplemented),
        }
    }
}
