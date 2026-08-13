use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum FSReturns {
    InvalidCommandFormat,
    UnsupportedMethod,
    InvalidHandle,
    UnknownPath,
    UnknownError,
    ResourceIsBusy,
    Ok,
}

impl From<FSReturns> for String {
    fn from(val: FSReturns) -> Self {
        match val {
            FSReturns::InvalidCommandFormat => "InvalidCommandFormat".to_string(),
            FSReturns::UnsupportedMethod => "UnsupportedMethod".to_string(),
            FSReturns::InvalidHandle => "InvalidHandle".to_string(),
            FSReturns::UnknownPath => "UnknownPath".to_string(),
            FSReturns::UnknownError => "UnknownError".to_string(),
            FSReturns::ResourceIsBusy => "ResourceIsBusy".to_string(),
            FSReturns::Ok => "Ok".to_string(),
        }
    }
}

impl From<FSReturns> for Vec<u8> {
    fn from(val: FSReturns) -> Self {
        match val {
            FSReturns::InvalidCommandFormat => vec![0x01],
            FSReturns::UnsupportedMethod => vec![0x02],
            FSReturns::InvalidHandle => vec![0x03],
            FSReturns::UnknownPath => vec![0x04],
            FSReturns::UnknownError => vec![0x05],
            FSReturns::ResourceIsBusy => vec![0x06],
            FSReturns::Ok => vec![0x07],
        }
    }
}
