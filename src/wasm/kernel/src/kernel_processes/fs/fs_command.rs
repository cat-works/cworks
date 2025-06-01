use super::{fs_obj::FSObjRef, fs_returns::FSReturns};

pub enum FSCommand {
    List(String),
    Stat(String),
    Get(String),
    Set(String, FSObjRef),
}

impl TryFrom<String> for FSCommand {
    type Error = FSReturns;
    fn try_from(value: String) -> Result<Self, FSReturns> {
        if value.is_empty() {
            return Err(FSReturns::InvalidCommandFormat);
        }

        let tokens = value.split('?').collect::<Vec<_>>();
        match (tokens.len(), tokens[0]) {
            (2, "List") => Ok(FSCommand::List(tokens[1].to_string())),
            (1, "Stat") => Ok(FSCommand::Stat(tokens[1].to_string())),
            (2, "Get") => Ok(FSCommand::Get(tokens[1].to_string())),
            // (3.., "Set") => Ok(FSCommand::Set(
            //     tokens[1].to_string(),
            //     IntrinsicFSObj::from_daemon_string(tokens[2..].join("?").to_string())?,
            // )),
            _ => Err(FSReturns::InvalidCommandFormat),
        }
    }
}
