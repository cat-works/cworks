#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum HandleData {
    IpcServer(String),
    IpcClient(String),
    None,
}
