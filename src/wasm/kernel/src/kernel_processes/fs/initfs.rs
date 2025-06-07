use super::fs_obj::{CompoundFSObj, FSObjRef};
use super::FSReturns;

fn usr(root: FSObjRef) -> Result<FSObjRef, FSReturns> {
    let usr: FSObjRef = CompoundFSObj::with_parent(root).into();

    usr.borrow_mut().add_child(
        "mime".to_string(),
        CompoundFSObj::with_parent(usr.clone()).into(),
    )?;
    usr.borrow_mut().add_child(
        "app".to_string(),
        CompoundFSObj::with_parent(usr.clone()).into(),
    )?;
    usr.borrow_mut().add_child(
        "ref".to_string(),
        CompoundFSObj::with_parent(usr.clone()).into(),
    )?;

    Ok(usr)
}

fn mnt(root: FSObjRef) -> FSObjRef {
    CompoundFSObj::with_parent(root).into()
}

fn workspace(root: FSObjRef) -> FSObjRef {
    CompoundFSObj::with_parent(root).into()
}

fn root() -> Result<FSObjRef, FSReturns> {
    let root: FSObjRef = CompoundFSObj::new().into();

    root.borrow_mut()
        .add_child("usr".to_string(), usr(root.clone())?)?;
    root.borrow_mut()
        .add_child("mnt".to_string(), mnt(root.clone()))?;
    root.borrow_mut()
        .add_child("workspace".to_string(), workspace(root.clone()))?;

    Ok(root)
}

pub fn initfs() -> FSObjRef {
    match root() {
        Ok(root) => root,
        Err(_) => {
            log::error!("Failed to initialize filesystem");
            CompoundFSObj::new().into() // Return an empty filesystem on error
        }
    }
}
