#[derive(Debug, Hash)]
pub enum Pid {
    UserProcess(u128),
    KernelProcess(u128),
}
