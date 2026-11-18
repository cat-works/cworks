#[derive(Clone, Debug, Default)]
pub enum PollResult {
    WaitForEvent,
    #[default]
    Pending,
    Done,
    Sleep(f32),
    WaitForProcess(u64),
    Root,
    GetPid,
}
