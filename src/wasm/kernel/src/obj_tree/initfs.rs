use crate::obj_tree::fs_obj::Object;

use super::fs_obj::FSObjRef;
use super::FSReturns;

fn usr(root: FSObjRef) -> Result<FSObjRef, FSReturns> {
    let usr: FSObjRef = FSObjRef::new_compound(root);

    usr.add_child("mime".to_string(), FSObjRef::new_compound(usr.clone()))?;
    usr.add_child("ref".to_string(), FSObjRef::new_compound(usr.clone()))?;

    Ok(usr)
}

fn mnt(root: FSObjRef) -> FSObjRef {
    FSObjRef::new_compound(root)
}

fn workspace(root: FSObjRef) -> FSObjRef {
    FSObjRef::new_compound(root)
}

fn root() -> Result<FSObjRef, FSReturns> {
    let root: FSObjRef = FSObjRef::empty_compound();

    root.add_child("usr".to_string(), usr(root.clone())?)?;
    root.add_child("mnt".to_string(), mnt(root.clone()))?;
    root.add_child("workspace".to_string(), workspace(root.clone()))?;

    Ok(root)
}

pub fn initfs() -> FSObjRef {
    match root() {
        Ok(root) => root,
        Err(_) => {
            log::error!("Failed to initialize filesystem");
            FSObjRef::empty_compound() // Return an empty filesystem on error
        }
    }
}
