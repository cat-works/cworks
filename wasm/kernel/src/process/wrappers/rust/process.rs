use crate::{process::wrappers::rust::ProcessSession, ProcessClient};

#[derive(Clone, Default)]
pub struct RustProcessCore(ProcessSession);

impl ProcessClient for RustProcessCore {
    fn get_session(&mut self) -> &mut ProcessSession {
        &mut self.0
    }
}
