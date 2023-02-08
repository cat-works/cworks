#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Handle {
    pub id: u128,
    pub pid: u128,
}

impl std::fmt::Display for Handle {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "H<{}>@P{}", self.id, self.pid)
    }
}

impl Handle {
    pub fn new(pid: u128, id: u128) -> Handle {
        Handle { pid, id }
    }
}
