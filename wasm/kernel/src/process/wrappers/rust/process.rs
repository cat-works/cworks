use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    obj_tree::FSObjRef, process::wrappers::rust::ProcessSession, ProcessClient, ProcessClientExt,
    SyscallData, SyscallError,
};

type DataHandler = Rc<Box<dyn Fn(Option<FSObjRef>) -> Result<(), SyscallError>>>;

#[derive(Clone, Default)]
pub struct RustProcessCore {
    session: ProcessSession,
    data_handlers: Rc<RefCell<HashMap<String, DataHandler>>>,
}

impl ProcessClient for RustProcessCore {
    fn get_session(&mut self) -> &mut ProcessSession {
        &mut self.session
    }

    fn handle_invocation(&mut self, data: &SyscallData) {
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
    }
}

impl RustProcessCore {
    pub async fn subscribe(
        &mut self,
        name: String,
        handler: DataHandler,
    ) -> Result<(), SyscallError> {
        self.data_handlers
            .borrow_mut()
            .insert(name.clone(), handler);

        ProcessClientExt::subscribe(self, name).await
    }
}
