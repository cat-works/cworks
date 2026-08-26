use crate::{
    obj_tree::FSObjRef, process::wrappers::rust::ProcessSession, PollResult, SyscallData,
    SyscallError,
};

pub trait ProcessClient {
    fn get_session(&mut self) -> &mut ProcessSession;
    fn handle_invocation(&mut self, data: &SyscallData);
}

pub trait ProcessClientExt {
    async fn sleep(&mut self, seconds: f32);
    async fn subscribe(&mut self, name: String) -> Result<(), SyscallError>;
    async fn unsubscribe(&mut self, name: String) -> Result<(), SyscallError>;
    async fn publish(&mut self, name: String, data: Option<FSObjRef>) -> Result<(), SyscallError>;
    async fn fs_list(&mut self, path: String) -> Result<(), SyscallError>;
    async fn fs_stat(&mut self, path: String) -> Result<(), SyscallError>;
    async fn fs_get(&mut self, path: String) -> Result<FSObjRef, SyscallError>;
    async fn fs_set(&mut self, path: String, data: FSObjRef) -> Result<(), SyscallError>;
    async fn fs_mkdir(&mut self, path: String, name: String) -> Result<(), SyscallError>;
    async fn wait_for_event(&mut self) -> Result<(), SyscallError>;
    fn pass_syscall_data(&mut self, data: &SyscallData);
    fn take_syscall(&mut self) -> PollResult;
}

impl<T: ProcessClient> ProcessClientExt for T {
    async fn sleep(&mut self, seconds: f32) {
        self.get_session()
            .do_syscall(PollResult::Sleep(seconds))
            .await;
    }

    async fn subscribe(&mut self, name: String) -> Result<(), SyscallError> {
        match self
            .get_session()
            .do_syscall(PollResult::Subscribe(name))
            .await
        {
            SyscallData::Fail(ref e) => Err(e.clone()),
            _ => Ok(()),
        }
    }

    async fn unsubscribe(&mut self, name: String) -> Result<(), SyscallError> {
        match self
            .get_session()
            .do_syscall(PollResult::Unsubscribe(name))
            .await
        {
            SyscallData::Fail(ref e) => Err(e.clone()),
            _ => Ok(()),
        }
    }

    async fn publish(&mut self, name: String, data: Option<FSObjRef>) -> Result<(), SyscallError> {
        match self
            .get_session()
            .do_syscall(PollResult::Publish(name, data))
            .await
        {
            SyscallData::Fail(ref e) => Err(e.clone()),
            _ => Ok(()),
        }
    }

    async fn fs_list(&mut self, path: String) -> Result<(), SyscallError> {
        match self.get_session().do_syscall(PollResult::List(path)).await {
            SyscallData::Fail(ref e) => Err(e.clone()),
            _ => Ok(()),
        }
    }

    async fn fs_stat(&mut self, path: String) -> Result<(), SyscallError> {
        match self.get_session().do_syscall(PollResult::Stat(path)).await {
            SyscallData::Fail(ref e) => Err(e.clone()),
            _ => Ok(()),
        }
    }

    async fn fs_get(&mut self, path: String) -> Result<FSObjRef, SyscallError> {
        match self.get_session().do_syscall(PollResult::Get(path)).await {
            SyscallData::FSGet(obj) => Ok(obj),
            SyscallData::Fail(ref e) => Err(e.clone()),
            _ => Err(SyscallError::NotImplemented),
        }
    }

    async fn fs_set(&mut self, path: String, data: FSObjRef) -> Result<(), SyscallError> {
        match self
            .get_session()
            .do_syscall(PollResult::Set(path, data))
            .await
        {
            SyscallData::Fail(ref e) => Err(e.clone()),
            _ => Ok(()),
        }
    }

    async fn fs_mkdir(&mut self, path: String, name: String) -> Result<(), SyscallError> {
        match self
            .get_session()
            .do_syscall(PollResult::Mkdir(path, name))
            .await
        {
            SyscallData::Fail(ref e) => Err(e.clone()),
            _ => Ok(()),
        }
    }

    async fn wait_for_event(&mut self) -> Result<(), SyscallError> {
        self.get_session()
            .do_syscall(PollResult::WaitForEvent)
            .await;
        Ok(())
    }

    fn pass_syscall_data(&mut self, data: &SyscallData) {
        self.handle_invocation(data);
        *self.get_session().response.borrow_mut() = Some(data.clone());
    }
    fn take_syscall(&mut self) -> PollResult {
        self.get_session().result.borrow_mut().clone()
    }
}
