#[derive(Debug, Clone, PartialEq)]
pub struct Handle {
    pub id: u128,
}

impl std::fmt::Display for Handle {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Handle[{}]", self.id)
    }
}

impl Handle {
    pub fn new(id: u128) -> Handle {
        Handle { id }
    }
}
