use std::ops::Deref;

use super::FSReturns;

pub struct DaemonString(String);

impl<T: Into<String>> From<T> for DaemonString {
    fn from(s: T) -> Self {
        DaemonString(s.into())
    }
}

impl Deref for DaemonString {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub trait DaemonCommunicable {
    fn to_daemon_string(&self) -> Result<DaemonString, FSReturns>;
    fn from_daemon_string(s: DaemonString) -> Result<Self, FSReturns>
    where
        Self: Sized;
}

impl<T: DaemonCommunicable> DaemonCommunicable for Box<T> {
    fn to_daemon_string(&self) -> Result<DaemonString, FSReturns> {
        self.as_ref().to_daemon_string()
    }

    fn from_daemon_string(s: DaemonString) -> Result<Self, FSReturns>
    where
        Self: Sized,
    {
        T::from_daemon_string(s).map(Box::new)
    }
}
