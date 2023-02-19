use std::collections::HashMap;

use crate::fs::{FSObj, RefOrVal};

fn usr() -> FSObj {
    let mut usr = HashMap::new();
    usr.insert(
        "mime".to_string(),
        FSObj::Dict(RefOrVal::Val(HashMap::new())),
    );
    usr.insert(
        "app".to_string(),
        FSObj::Dict(RefOrVal::Val(HashMap::new())),
    );
    usr.insert(
        "lib".to_string(),
        FSObj::Dict(RefOrVal::Val(HashMap::new())),
    );
    FSObj::Dict(RefOrVal::Val(usr))
}

pub fn initfs() -> FSObj {
    let mut root = HashMap::new();
    root.insert("usr".to_string(), usr());
    root.insert(
        "mnt".to_string(),
        FSObj::Dict(RefOrVal::Val(HashMap::new())),
    );
    root.insert(
        "workspace".to_string(),
        FSObj::Dict(RefOrVal::Val(HashMap::new())),
    );
    FSObj::Dict(RefOrVal::Val(root))
}
