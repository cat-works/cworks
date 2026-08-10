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
    fn from(value: FSReturns) -> Self {
        match value {
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
