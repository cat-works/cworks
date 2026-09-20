use std::{cell::RefCell, collections::HashMap, ops::Deref, rc::Rc};

use crate::{
    obj_tree::FSObjRef, process::wrappers::rust::ProcessSession, ProcessClient, ProcessClientExt,
    SyscallData, SyscallError,
};

type DataHandler = Rc<Box<dyn Fn(Option<FSObjRef>) -> Result<(), SyscallError>>>;

#[derive(Clone, Default)]
pub struct RustProcessCore {
    session: ProcessSession,
    data_handlers: Rc<RefCell<HashMap<*const <FSObjRef as Deref>::Target, DataHandler>>>,
}

impl ProcessClient for RustProcessCore {
    fn get_session(&mut self) -> &mut ProcessSession {
        &mut self.session
    }

    fn handle_invocation(&mut self, data: &SyscallData) {
        if let SyscallData::Invoke {
            caller_pid: _,
            obj,
            arg,
        } = data
        {
            if let Some(handler) = self
                .data_handlers
                .borrow()
                .iter()
                .find(|x| *x.0 == obj.as_ptr())
                .map(|x| x.1)
            {
                if let Err(e) = handler(arg.clone()) {
                    log::error!("Error handling data: {e:?}");
                }
            } else {
                log::warn!("No handler registered");
            }
        }
    }
}

impl RustProcessCore {
    pub async fn subscribe(
        &mut self,
        obj: FSObjRef,
        handler: DataHandler,
    ) -> Result<(), SyscallError> {
        self.data_handlers
            .borrow_mut()
            .insert(obj.as_ptr(), handler);

        ProcessClientExt::subscribe(self, obj).await
    }
}
