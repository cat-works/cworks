use std::sync::{Arc, Mutex};

use kernel::{Handle, Syscall, SyscallData, SyscallError};

use crate::dummy_future::DummyFuture;

pub struct Session {
    pub(crate) syscall: Arc<Mutex<Option<Syscall>>>,
    pub(crate) syscall_data: Arc<Mutex<SyscallData>>,
}

impl Default for Session {
    fn default() -> Self {
        let syscall = Arc::new(Mutex::new(Option::None));
        let data = Arc::new(Mutex::new(SyscallData::default()));
        Self {
            syscall: syscall,
            syscall_data: data,
        }
    }
}

impl Session {
    fn set_syscall(&self, syscall: Syscall) {
        *self.syscall.lock().unwrap() = Some(syscall);
    }
    fn return_handle(&self) -> Result<Arc<Handle>, SyscallError> {
        let m = self.syscall_data.lock().unwrap();
        match &(*m) {
            SyscallData::Handle(r) => {
                return r.clone();
            }
            _ => {
                println!("{:?}", m);
                panic!("unexpected syscall data");
            }
        }
    }

    pub fn get_syscall_data(&self) -> SyscallData {
        self.syscall_data.lock().unwrap().clone()
    }

    pub async fn ipc_create(&self, name: String) -> Result<Arc<Handle>, SyscallError> {
        self.set_syscall(Syscall::IpcCreate(name));
        DummyFuture::Started.await;
        self.return_handle()
    }
    pub async fn ipc_send(&self, handle: Arc<Handle>, data: String) -> Result<(), SyscallError> {
        self.set_syscall(Syscall::Send(handle, data));
        DummyFuture::Started.await;

        let m = self.syscall_data.lock().unwrap();
        match &(*m) {
            SyscallData::None => {
                return Ok(());
            }
            SyscallData::Handle(Err(e)) => {
                return Err(e.clone());
            }
            _ => {
                panic!("unexpected syscall data");
            }
        }
    }
    pub async fn ipc_connect(&self, name: String) -> Result<Arc<Handle>, SyscallError> {
        self.set_syscall(Syscall::IpcConnect(name));
        DummyFuture::Started.await;
        self.return_handle()
    }

    pub(crate) fn set_syscall_data(&self, data: &SyscallData) {
        *self.syscall_data.lock().unwrap() = data.clone();
    }
}
