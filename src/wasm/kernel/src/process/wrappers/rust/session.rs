use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

use crate::{Handle, Syscall, SyscallData, SyscallError};

use super::dummy_future::DummyFuture;

pub struct Session<T: Clone> {
    pub(crate) syscall: Arc<Mutex<Option<Syscall>>>,
    pub(crate) syscall_data: Arc<Mutex<SyscallData>>,

    data_buffer: Mutex<VecDeque<Arc<SyscallData>>>,

    value: T,
}

impl<T: Clone> Session<T> {
    pub fn new(value: T) -> Self {
        let syscall = Arc::new(Mutex::new(Option::None));
        let data = Arc::new(Mutex::new(SyscallData::default()));
        Self {
            syscall,
            syscall_data: data,
            data_buffer: VecDeque::new().into(),
            value,
        }
    }

    pub fn get_value(&self) -> T {
        self.value.clone()
    }

    fn poll_syscall_data(&self) {
        let m = self.syscall_data.lock().unwrap();
        match &(*m) {
            SyscallData::None => {}
            _ => {
                self.data_buffer
                    .lock()
                    .unwrap()
                    .push_back(Arc::new((*m).clone()));
            }
        }
    }

    fn set_syscall(&self, syscall: Syscall) {
        *self.syscall.lock().unwrap() = Some(syscall);
    }

    async fn return_handle(&self) -> Result<Handle, SyscallError> {
        loop {
            {
                let mut buffer = self.data_buffer.lock().unwrap();

                if let Some(x) = buffer.pop_front() {
                    match *x {
                        SyscallData::Handle(ref e) => return Ok(e.clone()),
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
                .lock()
                .unwrap()
                .pop_front()
                .map(|x| (*x).clone());
            if let Some(x) = f {
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

    pub async fn ipc_create(&self, name: String) -> Result<Handle, SyscallError> {
        self.set_syscall(Syscall::IpcCreate(name));
        DummyFuture::Started.await;
        self.return_handle().await
    }

    pub async fn ipc_send(&self, handle: Handle, data: String) -> Result<(), SyscallError> {
        self.set_syscall(Syscall::Send(handle, data));
        DummyFuture::Started.await;

        match *(self.syscall_data.lock().unwrap()) {
            SyscallData::Fail(ref e) => {
                self.set_syscall_data(&SyscallData::None);
                Err(e.clone())
            }
            _ => Ok(()),
        }
    }
    pub async fn ipc_connect(&self, name: String) -> Result<Handle, SyscallError> {
        self.set_syscall(Syscall::IpcConnect(name));
        DummyFuture::Started.await;
        self.return_handle().await
    }

    pub(crate) fn set_syscall_data(&self, data: &SyscallData) {
        *self.syscall_data.lock().unwrap() = data.clone();
    }
}
