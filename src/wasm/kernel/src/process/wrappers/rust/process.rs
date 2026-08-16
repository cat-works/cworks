use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{obj_tree::FSObjRef, PollResult, SyscallData, SyscallError};

use super::dummy_future::DummyFuture;

type DataHandler = Rc<Box<dyn Fn(Option<FSObjRef>) -> Result<(), SyscallError>>>;

#[derive(Clone, Default)]
pub struct RustProcessCore {
    pub(crate) result: Rc<RefCell<PollResult>>,
    pub(crate) syscall_data: Rc<RefCell<SyscallData>>,
    data_handlers: Rc<RefCell<HashMap<String, DataHandler>>>,
}

impl RustProcessCore {
    async fn do_syscall(&self, syscall: PollResult) {
        *self.result.borrow_mut() = syscall;
        DummyFuture::Started.await;
    }

    pub async fn sleep(&self, seconds: f32) {
        self.do_syscall(PollResult::Sleep(seconds)).await;
    }

    pub async fn subscribe(
        &mut self,
        name: String,
        handler: DataHandler,
    ) -> Result<(), SyscallError> {
        self.data_handlers
            .borrow_mut()
            .insert(name.clone(), handler);
        self.do_syscall(PollResult::Subscribe(name)).await;

        let m = self.syscall_data.borrow().clone();
        match m {
            SyscallData::FSSuccess => {
                self.set_syscall_data(&SyscallData::None);
                Ok(())
            }
            SyscallData::Fail(ref e) => {
                self.set_syscall_data(&SyscallData::None);
                Err(e.clone())
            }
            _ => Ok(()),
        }
    }

    pub async fn unsubscribe(&self, name: String) -> Result<(), SyscallError> {
        self.do_syscall(PollResult::Unsubscribe(name)).await;
        let m = self.syscall_data.borrow().clone();
        match m {
            SyscallData::FSSuccess => {
                self.set_syscall_data(&SyscallData::None);
                Ok(())
            }
            SyscallData::Fail(ref e) => {
                self.set_syscall_data(&SyscallData::None);
                Err(e.clone())
            }
            _ => Ok(()),
        }
    }

    pub async fn publish(&self, name: String, data: Option<FSObjRef>) -> Result<(), SyscallError> {
        self.do_syscall(PollResult::Publish(name, data)).await;
        Ok(())
    }

    pub async fn fs_list(&self, path: String) -> Result<(), SyscallError> {
        self.do_syscall(PollResult::List(path)).await;

        let m = self.syscall_data.borrow().clone();
        match m {
            SyscallData::FSList(_) => {
                self.set_syscall_data(&SyscallData::None);
                Ok(())
            }
            SyscallData::Fail(ref e) => {
                self.set_syscall_data(&SyscallData::None);
                Err(e.clone())
            }
            _ => Ok(()),
        }
    }

    pub async fn fs_stat(&self, path: String) -> Result<(), SyscallError> {
        self.do_syscall(PollResult::Stat(path)).await;

        let m = self.syscall_data.borrow().clone();
        match m {
            SyscallData::FSStat(_) => {
                self.set_syscall_data(&SyscallData::None);
                Ok(())
            }
            SyscallData::Fail(ref e) => {
                self.set_syscall_data(&SyscallData::None);
                Err(e.clone())
            }
            _ => Ok(()),
        }
    }

    pub async fn fs_get(&self, path: String) -> Result<(), SyscallError> {
        self.do_syscall(PollResult::Get(path)).await;

        let m = self.syscall_data.borrow().clone();
        match m {
            SyscallData::FSGet(_) => {
                self.set_syscall_data(&SyscallData::None);
                Ok(())
            }
            SyscallData::Fail(ref e) => {
                self.set_syscall_data(&SyscallData::None);
                Err(e.clone())
            }
            _ => Ok(()),
        }
    }

    pub async fn fs_set(&self, path: String, data: FSObjRef) -> Result<(), SyscallError> {
        self.do_syscall(PollResult::Set(path, data)).await;

        let m = self.syscall_data.borrow().clone();
        match m {
            SyscallData::FSSuccess => {
                self.set_syscall_data(&SyscallData::None);
                Ok(())
            }
            SyscallData::Fail(ref e) => {
                self.set_syscall_data(&SyscallData::None);
                Err(e.clone())
            }
            _ => Ok(()),
        }
    }

    pub async fn fs_mkdir(&self, path: String, name: String) -> Result<(), SyscallError> {
        self.do_syscall(PollResult::Mkdir(path, name)).await;

        let m = self.syscall_data.borrow().clone();
        match m {
            SyscallData::FSSuccess => {
                self.set_syscall_data(&SyscallData::None);
                Ok(())
            }
            SyscallData::Fail(ref e) => {
                self.set_syscall_data(&SyscallData::None);
                Err(e.clone())
            }
            _ => Ok(()),
        }
    }

    pub async fn wait_for_event(&self) -> Result<(), SyscallError> {
        self.do_syscall(PollResult::WaitForEvent).await;
        Ok(())
    }

    pub(crate) fn set_syscall_data(&self, data: &SyscallData) {
        if let SyscallData::Invoke {
            caller_pid: _,
            path,
            arg,
        } = data
        {
            if let Some(handler) = self.data_handlers.borrow().get(path) {
                if let Err(e) = handler(arg.clone()) {
                    log::error!("Error handling data for path {path}: {e:?}");
                }
            } else {
                log::warn!("No handler registered for path: {path}");
            }
        }

        *self.syscall_data.borrow_mut() = data.clone();
    }
}
