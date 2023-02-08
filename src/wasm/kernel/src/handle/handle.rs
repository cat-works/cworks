use super::HandleData;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Handle {
    pub id: u128,
    pub pid: u128,
    pub(crate) data: HandleData,
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
