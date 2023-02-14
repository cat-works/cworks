use super::HandleData;

#[derive(Debug, Clone)]
pub struct HandleCore {
    pub id: u128,
    pub pid: u128,
    pub(crate) data: HandleData,
}
