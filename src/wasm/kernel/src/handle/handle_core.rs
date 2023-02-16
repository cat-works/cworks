use super::HandleData;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct HandleCore {
    pub id: u128,
    pub pid: u128,

    #[serde(skip_serializing)]
    pub(crate) data: HandleData,
}
