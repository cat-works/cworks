use std::sync::{Arc, Mutex};

use kernel::{Handle, Syscall, SyscallData};

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
    pub async fn ipc_create(&self, name: String) -> Arc<Handle> {
        self.syscall
            .lock()
            .unwrap()
            .replace(Syscall::IpcCreate(name));
        DummyFuture::Started.await;

        let m = self.syscall_data.lock().unwrap();
        match &(*m) {
            SyscallData::Handle(Ok(h)) => {
                return h.clone();
            }
            _ => {
                panic!("unexpected syscall data");
            }
        }
    }
    pub(crate) fn set_syscall_data(&self, data: &SyscallData) {
        *self.syscall_data.lock().unwrap() = data.clone();
    }
}
