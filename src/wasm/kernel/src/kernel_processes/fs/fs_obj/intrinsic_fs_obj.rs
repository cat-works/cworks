use crate::fs::{
    traits::{DaemonCommunicable, DaemonString},
    FSReturns,
};
use std::fmt::{Debug, Display};

use super::{
    object::{FileKind, FileStat},
    FSObjRef, Object,
};

#[derive(Debug)]
pub enum IntrinsicFSObj {
    Int(i128),
    String(String),
    Boolean(bool),
    Float(f32),
    Double(f64),
    Bytes(Vec<u8>),
    Null,
}

impl Display for IntrinsicFSObj {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IntrinsicFSObj::Int(x) => write!(f, "{x}"),
            IntrinsicFSObj::String(x) => write!(f, "\"{x}\""),
            IntrinsicFSObj::Boolean(x) => write!(f, "{x:#}"),
            IntrinsicFSObj::Float(x) => write!(f, "{x}"),
            IntrinsicFSObj::Double(x) => write!(f, "{x}"),
            IntrinsicFSObj::Bytes(x) => write!(f, "{x:?}"),
            IntrinsicFSObj::Null => write!(f, "Null"),
        }
    }
}

impl DaemonCommunicable for IntrinsicFSObj {
    fn to_daemon_string(&self) -> Result<DaemonString, FSReturns> {
        match self {
            IntrinsicFSObj::Boolean(x) => Ok(format!("Boolean?{}", x).into()),
            IntrinsicFSObj::Float(x) => Ok(format!("Float?{}", x).into()),
            IntrinsicFSObj::Int(x) => Ok(format!("Integer?{}", x).into()),
            IntrinsicFSObj::Double(x) => Ok(format!("Double?{}", x).into()),
            IntrinsicFSObj::String(x) => Ok(format!("String?{}", x).into()),
            IntrinsicFSObj::Bytes(x) => Ok(format!(
                "Bytes?{}",
                x.iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>()
                    .join("?"),
            )
            .into()),
            IntrinsicFSObj::Null => Ok("Null".into()),
        }
    }
    fn from_daemon_string(s: DaemonString) -> Result<Self, FSReturns>
    where
        Self: Sized,
    {
        let tokens = s.split('?').collect::<Vec<_>>();

        if tokens.len() < 2 {
            return Err(FSReturns::InvalidCommandFormat);
        }

        let obj = match tokens[0] {
            "Boolean" => Ok(Self::Boolean(
                tokens[1]
                    .parse::<bool>()
                    .map_err(|_| FSReturns::InvalidCommandFormat)?,
            )),
            "Float" => Ok(Self::Float(
                tokens[1]
                    .parse::<f32>()
                    .map_err(|_| FSReturns::InvalidCommandFormat)?,
            )),
            "Integer" => Ok(Self::Int(
                tokens[1]
                    .parse::<i128>()
                    .map_err(|_| FSReturns::InvalidCommandFormat)?,
            )),
            "Double" => Ok(Self::Double(
                tokens[1]
                    .parse::<f64>()
                    .map_err(|_| FSReturns::InvalidCommandFormat)?,
            )),
            "String" => Ok(Self::String(tokens[1..].join("?").to_string())),
            "Bytes" => Ok(Self::Bytes(
                tokens[1..]
                    .iter()
                    .map(|x| x.parse::<u8>().map_err(|_| FSReturns::InvalidCommandFormat))
                    .collect::<Result<Vec<_>, FSReturns>>()?,
            )),
            "Null" => Ok(Self::Null),
            _ => Err(FSReturns::InvalidCommandFormat),
        };

        obj
    }
}

impl Object for IntrinsicFSObj {
    fn get_obj(&self, _part: String) -> Result<FSObjRef, FSReturns> {
        Err(FSReturns::UnknownPath)
    }

    fn stat(&self) -> Result<super::object::FileStat, FSReturns> {
        Ok(FileStat {
            kind: FileKind::File,
        })
    }
}
