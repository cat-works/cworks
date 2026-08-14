use std::{cell::RefCell, collections::VecDeque, rc::Rc};

use crate::{handle::HandleRef, obj_tree::FSObjRef, Syscall, SyscallData, SyscallError};

use super::dummy_future::DummyFuture;

#[derive(Clone, Default)]
pub struct RustProcessCore {
    pub(crate) syscall: Rc<RefCell<Option<Syscall>>>,
    pub(crate) syscall_data: Rc<RefCell<SyscallData>>,

    data_buffer: RefCell<VecDeque<Rc<SyscallData>>>,
}

impl RustProcessCore {
    fn poll_syscall_data(&self) {
        let m = self.syscall_data.borrow().clone();
        match m {
            SyscallData::None => {}
            _ => {
                log::trace!("RustProcessCore::poll_syscall_data: pushing data to buffer: {m:?}",);
                self.data_buffer
                    .borrow_mut()
                    .push_back(Rc::new((m).clone()));
            }
        }
    }

    fn set_syscall(&self, syscall: Syscall) {
        *self.syscall.borrow_mut() = Some(syscall);
    }

    async fn return_handle(&self) -> Result<HandleRef, SyscallError> {
        loop {
            {
                let mut buffer = self.data_buffer.borrow_mut();

                if let Some(x) = buffer.pop_front() {
                    match *x {
                        SyscallData::Handle(ref e) => {
                            log::trace!(
                                "RustProcessCore::return_handle: returning handle from buffer: {e:?}",
                            );
                            return Ok(*e);
                        }
                        SyscallData::Fail(ref e) => {
                            log::trace!(
                                "RustProcessCore::return_handle: returning error from buffer: {e:?}",
                            );
                            return Err(e.clone());
                        }
                        _ => {
                            buffer.push_back(x);
                        }
                    }
                }
            }

            self.poll_syscall_data();
            DummyFuture::Started.await;
        }
    }

    pub async fn get_syscall_data(&self) -> SyscallData {
        loop {
            let f = self
                .data_buffer
                .borrow_mut()
                .pop_front()
                .map(|x| (*x).clone());
            if let Some(x) = f {
                log::trace!(
                    "RustProcessCore::get_syscall_data: returning data from buffer: {:?}",
                    x
                );
                return x;
            } else {
                self.poll_syscall_data();
                DummyFuture::Started.await;
            }
        }
    }

    pub async fn sleep(&self, seconds: f32) {
        self.set_syscall(Syscall::Sleep(seconds));
        DummyFuture::Started.await;
    }

    pub async fn ipc_create(&self, name: String) -> Result<HandleRef, SyscallError> {
        self.set_syscall(Syscall::IpcCreate(name));
        DummyFuture::Started.await;
        self.return_handle().await
    }

    pub async fn ipc_send(&self, handle: HandleRef, data: String) -> Result<(), SyscallError> {
        self.set_syscall(Syscall::Send(handle, data));
        DummyFuture::Started.await;

        let m = self.syscall_data.borrow().clone();
        match m {
            SyscallData::Fail(ref e) => {
                self.set_syscall_data(&SyscallData::None);
                Err(e.clone())
            }
            _ => Ok(()),
        }
    }
    pub async fn ipc_connect(&self, name: String) -> Result<HandleRef, SyscallError> {
        self.set_syscall(Syscall::IpcConnect(name));
        DummyFuture::Started.await;
        self.return_handle().await
    }

    pub async fn fs_list(&self, path: String) -> Result<(), SyscallError> {
        self.set_syscall(Syscall::List(path));
        DummyFuture::Started.await;

        let m = self.syscall_data.borrow().clone();
        match m {
            SyscallData::FSList(r) => {
                self.set_syscall_data(&SyscallData::None);
                log::trace!("RustProcessCore::fs_list: returning FSList: {:?}", r);
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
        self.set_syscall(Syscall::Stat(path));
        DummyFuture::Started.await;

        let m = self.syscall_data.borrow().clone();
        match m {
            SyscallData::FSStat(r) => {
                self.set_syscall_data(&SyscallData::None);
                log::trace!("RustProcessCore::fs_stat: returning FSStat: {:?}", r);
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
        self.set_syscall(Syscall::Get(path));
        DummyFuture::Started.await;

        let m = self.syscall_data.borrow().clone();
        match m {
            SyscallData::FSGet(obj) => {
                self.set_syscall_data(&SyscallData::None);
                log::trace!("RustProcessCore::fs_get: returning FSGet: {:?}", obj);
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
        self.set_syscall(Syscall::Set(path, data));
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
        self.set_syscall(Syscall::Mkdir(path, name));
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
