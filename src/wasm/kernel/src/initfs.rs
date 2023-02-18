use std::collections::HashMap;

use crate::fs::{FSObj, RefOrVal};

pub fn initfs() -> FSObj {
    let mut root = HashMap::new();
    root.insert(
        "usr".to_string(),
        FSObj::Dist(RefOrVal::Val(HashMap::new())),
    );
    FSObj::Dist(RefOrVal::Val(root))
}
