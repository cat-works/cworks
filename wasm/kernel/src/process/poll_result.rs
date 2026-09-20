use crate::obj_tree::FSObjRef;

#[derive(Clone, Debug, Default)]
pub enum PollResult {
    WaitForEvent,
    #[default]
    Pending,
    Done,
    Sleep(f32),
    WaitForProcess(u64),
    Root,
    Subscribe(FSObjRef),
    Unsubscribe(FSObjRef),
    Publish(FSObjRef, Option<FSObjRef>),
    GetPid,
}
