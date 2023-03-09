use std::{collections::HashMap, sync::Arc};

use crate::fs::FSObj;

fn usr_mime_cafecode() -> FSObj {
    let mut root = HashMap::new();
    root.insert(
        "text".to_string(),
        FSObj::String(Arc::new("text/x-cafecode".to_string())),
    );
    root.insert(
        "icon".to_string(),
        FSObj::String(Arc::new("/usr/share/icons/cafecode.png".to_string())),
    );
    root.insert(
        "handler".to_string(),
        FSObj::String(Arc::new("0sys_wu/cafecode".to_string())),
    );
    FSObj::Dict(Arc::new(root.into()))
}

fn usr_mime() -> FSObj {
    let mut root = HashMap::new();
    root.insert("cafecode".to_string(), usr_mime_cafecode());
    FSObj::Dict(Arc::new(root.into()))
}

fn usr() -> FSObj {
    let mut usr = HashMap::new();
    usr.insert("mime".to_string(), usr_mime());
    usr.insert(
        "app".to_string(),
        FSObj::Dict(Arc::new(HashMap::new().into())),
    );
    usr.insert(
        "lib".to_string(),
        FSObj::Dict(Arc::new(HashMap::new().into())),
    );
    FSObj::Dict(Arc::new(usr.into()))
}

pub fn initfs() -> FSObj {
    let mut root = HashMap::new();
    root.insert("usr".to_string(), usr());
    root.insert(
        "mnt".to_string(),
        FSObj::Dict(Arc::new(HashMap::new().into())),
    );
    root.insert(
        "workspace".to_string(),
        FSObj::Dict(Arc::new(HashMap::new().into())),
    );
    FSObj::Dict(Arc::new(root.into()))
}
