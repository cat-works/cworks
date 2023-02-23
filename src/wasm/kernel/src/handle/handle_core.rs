use super::HandleData;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandleCore {
    pub id: u128,
    pub pid: u128,

    #[serde(skip_serializing, skip_deserializing)]
    pub(crate) data: HandleData,
}
