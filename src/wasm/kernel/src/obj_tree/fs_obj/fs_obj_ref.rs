use std::{cell::RefCell, fmt::Debug, ops::Deref, rc::Rc};

use serde::{Deserialize, Serialize};

use crate::obj_tree::{traits::DaemonString, DaemonCommunicable, IntrinsicFSObj};

use super::Object;

pub struct FSObjRef(Rc<RefCell<Box<dyn Object>>>);

impl<T: Object + 'static> From<T> for FSObjRef {
    fn from(obj: T) -> Self {
        FSObjRef(Rc::new(RefCell::new(Box::new(obj))))
    }
}

impl Serialize for FSObjRef {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let obj = self.0.borrow().to_daemon_string();
        match obj {
            Ok(s) => serializer.serialize_str(&s),
            Err(_e) => Err(serde::ser::Error::custom(
                "Failed to serialize FSObjRef".to_string(),
            )),
        }
    }
}

impl<'de> Deserialize<'de> for FSObjRef {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: DaemonString = String::deserialize(deserializer)?.into();
        let obj = IntrinsicFSObj::from_daemon_string(s);
        match obj {
            Ok(o) => Ok(FSObjRef(Rc::new(RefCell::new(Box::new(o))))),
            Err(_e) => Err(serde::de::Error::custom(
                "Failed to deserialize FSObjRef".to_string(),
            )),
        }
    }
}

impl Debug for FSObjRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FSObjRef({:?})", self.0)
    }
}

impl Clone for FSObjRef {
    fn clone(&self) -> Self {
        FSObjRef(self.0.clone())
    }
}

impl Deref for FSObjRef {
    type Target = Rc<RefCell<Box<dyn Object>>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
