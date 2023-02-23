use std::{ops::Deref, sync::Arc};

use serde::{Deserialize, Serialize};

use crate::kernel_processes::fs::FSObj;

use super::{handle_core::HandleCore, HandleData};

#[derive(Debug)]
pub struct Handle(Arc<HandleCore>);

impl Serialize for Handle {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}
impl<'de> Deserialize<'de> for Handle {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        HandleCore::deserialize(deserializer).map(|x| Self(Arc::new(x)))
    }
    fn deserialize_in_place<D>(deserializer: D, place: &mut Self) -> Result<(), D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        *place = Self::deserialize(deserializer)?;
        Ok(())
    }
}

impl Clone for Handle {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl Deref for Handle {
    type Target = HandleCore;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl PartialEq for Handle {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.pid == other.pid
    }
}

impl From<Handle> for FSObj {
    fn from(x: Handle) -> FSObj {
        FSObj::Handle(x)
    }
}

impl std::fmt::Display for Handle {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Handle<{}>@P{} {{{}}}", self.id, self.pid, self.data)
    }
}

impl Default for Handle {
    fn default() -> Self {
        Self::new(0, 0, HandleData::None)
    }
}

impl Handle {
    pub fn new(pid: u128, id: u128, data: HandleData) -> Self {
        Self(Arc::new(HandleCore { pid, id, data }))
    }
}
