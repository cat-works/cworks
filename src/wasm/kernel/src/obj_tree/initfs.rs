use super::{FSObjRef, FSReturns};

fn usr(root: FSObjRef) -> Result<FSObjRef, FSReturns> {
    let usr: FSObjRef = FSObjRef::new_compound(root);

    usr.add_child("mime", &FSObjRef::new_compound(usr.clone()))?;
    usr.add_child("ref", &FSObjRef::new_compound(usr.clone()))?;

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

    root.add_child("usr", &usr(root.clone())?)?;
    root.add_child("mnt", &mnt(root.clone()))?;
    root.add_child("workspace", &workspace(root.clone()))?;

    Ok(root)
}

pub fn initfs() -> FSObjRef {
    root().unwrap_or_else(|_| {
        log::error!("Failed to initialize filesystem");
        FSObjRef::empty_compound() // Return an empty filesystem on error
    })
}
