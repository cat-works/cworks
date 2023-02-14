use std::{ops::Deref, sync::Arc};

use serde::{ser::SerializeMap, Deserialize, Serialize};

use crate::fs::FSObj;

use super::{handle_core::HandleCore, HandleData};

#[derive(Debug)]
pub struct Handle(Arc<HandleCore>);

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

impl Serialize for Handle {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut m = serializer.serialize_map(Some(3))?;
        m.serialize_entry("id", &self.id)?;
        m.serialize_entry("pid", &self.pid)?;

        m.end()
    }
}

impl<'de> Deserialize<'de> for Handle {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Field {
            Id,
            Pid,
        }

        struct HandleVisitor;

        impl<'de> serde::de::Visitor<'de> for HandleVisitor {
            type Value = Handle;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct Handle")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: serde::de::MapAccess<'de>,
            {
                let mut id = None;
                let mut pid = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Id => {
                            if id.is_some() {
                                return Err(serde::de::Error::duplicate_field("id"));
                            }
                            id = Some(map.next_value()?);
                        }
                        Field::Pid => {
                            if pid.is_some() {
                                return Err(serde::de::Error::duplicate_field("pid"));
                            }
                            pid = Some(map.next_value()?);
                        }
                    }
                }

                let id = id.ok_or_else(|| serde::de::Error::missing_field("id"))?;
                let pid = pid.ok_or_else(|| serde::de::Error::missing_field("pid"))?;

                Ok(Handle::new(pid, id, HandleData::None))
            }
        }

        const FIELDS: &'static [&'static str] = &["id", "pid"];
        deserializer.deserialize_struct("Handle", FIELDS, HandleVisitor)
    }
}

impl PartialEq for Handle {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.pid == other.pid
    }
}

impl From<Handle> for FSObj {
    fn from(x: Handle) -> FSObj {
        FSObj::Handle(crate::fs::RefOrVal::Val(x))
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
