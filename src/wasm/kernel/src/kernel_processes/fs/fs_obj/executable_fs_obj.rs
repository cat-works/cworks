// ExecutableFSObj: 実行可能ファイルを表すFSオブジェクト
// ここではシンプルにスクリプト(文字列)を格納する例

use crate::fs::{
    traits::{DaemonCommunicable, DaemonString},
    FSReturns,
};
use std::fmt::{Debug, Display};

use super::{
    object::{FileKind, FileStat},
    FSObjRef, Object,
};

#[derive(Debug, Clone)]
pub struct ExecutableFSObj {
    pub script: String, // 任意のスクリプトやコマンド列
}

impl ExecutableFSObj {
    pub fn new(script: String) -> Self {
        Self { script }
    }
}

impl Display for ExecutableFSObj {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Executable: {}", self.script)
    }
}

impl DaemonCommunicable for ExecutableFSObj {
    fn to_daemon_string(&self) -> Result<DaemonString, FSReturns> {
        Ok(format!("Executable?{}", self.script).into())
    }
    fn from_daemon_string(s: DaemonString) -> Result<Self, FSReturns>
    where
        Self: Sized,
    {
        let tokens = s.splitn(2, '?').collect::<Vec<_>>();
        if tokens.len() != 2 {
            return Err(FSReturns::InvalidCommandFormat);
        }
        Ok(Self {
            script: tokens[1].to_string(),
        })
    }
}

impl Object for ExecutableFSObj {
    fn stat(&self) -> Result<FileStat, FSReturns> {
        Ok(FileStat {
            kind: FileKind::File,
        })
    }
    fn get_obj(&self, _part: String) -> Result<FSObjRef, FSReturns> {
        Err(FSReturns::UnknownPath)
    }
}
