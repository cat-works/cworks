use std::{rc::Rc, sync::Arc};

use crate::fs::FSObj;

use super::HandleData;

#[derive(Debug, Clone)]
pub struct Handle {
    pub id: u128,
    pub pid: u128,
    pub(crate) data: HandleData,
}

impl PartialEq for Handle {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.pid == other.pid
    }

    fn ne(&self, other: &Self) -> bool {
        self.id != other.id || self.pid != other.pid
    }
}

impl From<Arc<Handle>> for FSObj {
    fn from(x: Arc<Handle>) -> FSObj {
        FSObj::Handle(crate::fs::RefOrVal::Val(x))
    }
}

impl std::fmt::Display for Handle {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "H<{}>@P{}", self.id, self.pid)
    }
}

impl Default for Handle {
    fn default() -> Self {
        Self::new(0, 0, HandleData::None)
    }
}

impl Handle {
    pub fn new(pid: u128, id: u128, data: HandleData) -> Handle {
        Handle { pid, id, data }
    }
}
