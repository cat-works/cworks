use serde::{Deserialize, Serialize};

use crate::obj_tree::FSObjRef;

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub enum PollResult {
    #[default]
    Pending,
    Done(i64),
    Sleep(f32),
    WaitForProcess(u128),
    List(String),
    Stat(String),
    Get(String),
    Set(String, FSObjRef),
    Mkdir(String, String),
    Subscribe(String),
    Unsubscribe(String),
    Publish(String, Option<FSObjRef>),
}
