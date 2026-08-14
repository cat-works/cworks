use std::{cell::RefCell, collections::VecDeque, rc::Rc};

use crate::{obj_tree::FSObjRef, PollResult, SyscallData, SyscallError};

use super::dummy_future::DummyFuture;

#[derive(Clone, Default)]
pub struct RustProcessCore {
    pub(crate) result: Rc<RefCell<PollResult>>,
    pub(crate) syscall_data: Rc<RefCell<SyscallData>>,

    data_buffer: RefCell<VecDeque<Rc<SyscallData>>>,
}

impl RustProcessCore {
    fn poll_syscall_data(&self) {
        let m = self.syscall_data.borrow().clone();
        if !matches!(m, SyscallData::None) {
            log::trace!("RustProcessCore::poll_syscall_data: pushing data to buffer: {m:?}");
            self.data_buffer.borrow_mut().push_back(Rc::new(m));
        }
    }

    fn set_syscall(&self, syscall: PollResult) {
        *self.result.borrow_mut() = syscall;
    }

    pub async fn get_syscall_data(&self) -> SyscallData {
        loop {
            let f = self
                .data_buffer
                .borrow_mut()
                .pop_front()
                .map(|x| (*x).clone());
            if let Some(x) = f {
                log::trace!("RustProcessCore::get_syscall_data: returning data from buffer: {x:?}");
                return x;
            }
            self.poll_syscall_data();
            DummyFuture::Started.await;
        }
    }

    pub async fn sleep(&self, seconds: f32) {
        self.set_syscall(PollResult::Sleep(seconds));
        DummyFuture::Started.await;
    }

    pub async fn subscribe(&self, name: String) -> Result<(), SyscallError> {
        self.set_syscall(PollResult::Subscribe(name));
        DummyFuture::Started.await;
        let m = self.syscall_data.borrow().clone();
        match m {
            SyscallData::FSSuccess => {
                self.set_syscall_data(&SyscallData::None);
                log::trace!("RustProcessCore::fs_set: returning FSSuccess");
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
        self.set_syscall(PollResult::Unsubscribe(name));
        DummyFuture::Started.await;
        let m = self.syscall_data.borrow().clone();
        match m {
            SyscallData::FSSuccess => {
                self.set_syscall_data(&SyscallData::None);
                log::trace!("RustProcessCore::fs_set: returning FSSuccess");
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
        self.set_syscall(PollResult::Publish(name, data));
        DummyFuture::Started.await;
        Ok(())
    }

    pub async fn fs_list(&self, path: String) -> Result<(), SyscallError> {
        self.set_syscall(PollResult::List(path));
        DummyFuture::Started.await;

        let m = self.syscall_data.borrow().clone();
        match m {
            SyscallData::FSList(r) => {
                self.set_syscall_data(&SyscallData::None);
                log::trace!("RustProcessCore::fs_list: returning FSList: {r:?}");
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
        self.set_syscall(PollResult::Stat(path));
        DummyFuture::Started.await;

        let m = self.syscall_data.borrow().clone();
        match m {
            SyscallData::FSStat(r) => {
                self.set_syscall_data(&SyscallData::None);
                log::trace!("RustProcessCore::fs_stat: returning FSStat: {r:?}");
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
        self.set_syscall(PollResult::Get(path));
        DummyFuture::Started.await;

        let m = self.syscall_data.borrow().clone();
        match m {
            SyscallData::FSGet(obj) => {
                self.set_syscall_data(&SyscallData::None);
                log::trace!("RustProcessCore::fs_get: returning FSGet: {obj:?}");
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
        self.set_syscall(PollResult::Set(path, data));
        DummyFuture::Started.await;

        let m = self.syscall_data.borrow().clone();
        match m {
            SyscallData::FSSuccess => {
                self.set_syscall_data(&SyscallData::None);
                log::trace!("RustProcessCore::fs_set: returning FSSuccess");
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
        self.set_syscall(PollResult::Mkdir(path, name));
        DummyFuture::Started.await;

        let m = self.syscall_data.borrow().clone();
        match m {
            SyscallData::FSSuccess => {
                self.set_syscall_data(&SyscallData::None);
                log::trace!("RustProcessCore::fs_mkdir: returning FSSuccess");
                Ok(())
            }
            SyscallData::Fail(ref e) => {
                self.set_syscall_data(&SyscallData::None);
                Err(e.clone())
            }
            _ => Ok(()),
        }
    }

    pub(crate) fn set_syscall_data(&self, data: &SyscallData) {
        *self.syscall_data.borrow_mut() = data.clone();
    }
}
