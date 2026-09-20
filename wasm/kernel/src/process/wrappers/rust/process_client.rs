use crate::{
    obj_tree::FSObjRef, process::wrappers::rust::ProcessSession, PollResult, SyscallData,
    SyscallError,
};

pub trait ProcessClient {
    fn get_session(&mut self) -> &mut ProcessSession;
    fn handle_invocation(&mut self, data: &SyscallData);
}

pub trait ProcessClientExt {
    fn sleep(&mut self, seconds: f32) -> impl std::future::Future<Output = ()>;
    fn subscribe(
        &mut self,
        obj: FSObjRef,
    ) -> impl std::future::Future<Output = Result<(), SyscallError>>;
    fn unsubscribe(
        &mut self,
        obj: FSObjRef,
    ) -> impl std::future::Future<Output = Result<(), SyscallError>>;
    fn publish(
        &mut self,
        obj: FSObjRef,
        data: Option<FSObjRef>,
    ) -> impl std::future::Future<Output = Result<(), SyscallError>>;
    fn fs_root(&mut self) -> impl std::future::Future<Output = Result<FSObjRef, SyscallError>>;
    fn wait_for_event(&mut self) -> impl std::future::Future<Output = Result<(), SyscallError>>;
    fn get_pid(&mut self) -> impl std::future::Future<Output = Result<u64, SyscallError>>;
    fn pass_syscall_data(&mut self, data: &SyscallData);
    fn take_syscall(&mut self) -> PollResult;
}

impl<T: ProcessClient> ProcessClientExt for T {
    async fn sleep(&mut self, seconds: f32) {
        self.get_session()
            .do_syscall(PollResult::Sleep(seconds))
            .await;
    }

    async fn subscribe(&mut self, obj: FSObjRef) -> Result<(), SyscallError> {
        match self
            .get_session()
            .do_syscall(PollResult::Subscribe(obj))
            .await
        {
            SyscallData::Fail(ref e) => Err(e.clone()),
            _ => Ok(()),
        }
    }

    async fn unsubscribe(&mut self, obj: FSObjRef) -> Result<(), SyscallError> {
        match self
            .get_session()
            .do_syscall(PollResult::Unsubscribe(obj))
            .await
        {
            SyscallData::Fail(ref e) => Err(e.clone()),
            _ => Ok(()),
        }
    }

    async fn publish(&mut self, obj: FSObjRef, data: Option<FSObjRef>) -> Result<(), SyscallError> {
        match self
            .get_session()
            .do_syscall(PollResult::Publish(obj, data))
            .await
        {
            SyscallData::Fail(ref e) => Err(e.clone()),
            _ => Ok(()),
        }
    }

    async fn fs_root(&mut self) -> Result<FSObjRef, SyscallError> {
        match self.get_session().do_syscall(PollResult::Root).await {
            SyscallData::Fail(ref e) => Err(e.clone()),
            SyscallData::FSRoot(e) => Ok(e),
            _ => Err(SyscallError::UnreachableEntry),
        }
    }

    async fn wait_for_event(&mut self) -> Result<(), SyscallError> {
        self.get_session()
            .do_syscall(PollResult::WaitForEvent)
            .await;
        Ok(())
    }

    async fn get_pid(&mut self) -> Result<u64, SyscallError> {
        match self.get_session().do_syscall(PollResult::GetPid).await {
            SyscallData::GetPid(pid) => Ok(pid),
            SyscallData::Fail(ref e) => Err(e.clone()),
            _ => Err(SyscallError::NotImplemented),
        }
    }

    fn pass_syscall_data(&mut self, data: &SyscallData) {
        self.handle_invocation(data);
        *self.get_session().response.borrow_mut() = Some(data.clone());
    }
    fn take_syscall(&mut self) -> PollResult {
        self.get_session().result.borrow_mut().clone()
    }
}
