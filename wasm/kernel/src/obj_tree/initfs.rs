use crate::SyscallError;

use super::FSObjRef;

fn usr(root: &FSObjRef) -> Result<FSObjRef, SyscallError> {
    let usr = FSObjRef::new_compound(root);

    usr.add_child("mime", &FSObjRef::new_compound(&usr))?;
    usr.add_child("ref", &FSObjRef::new_compound(&usr))?;
    usr.add_child("bin", &FSObjRef::new_compound(&usr))?;
    usr.add_child("lib", &FSObjRef::new_compound(&usr))?;

    Ok(usr)
}

fn mnt(root: &FSObjRef) -> FSObjRef {
    FSObjRef::new_compound(root)
}

fn workspace(root: &FSObjRef) -> FSObjRef {
    FSObjRef::new_compound(root)
}

fn root() -> Result<FSObjRef, SyscallError> {
    let root = FSObjRef::empty_compound();

    root.add_child("usr", &usr(&root)?)?;
    root.add_child("mnt", &mnt(&root))?;
    root.add_child("workspace", &workspace(&root))?;

    Ok(root)
}

pub fn initfs() -> FSObjRef {
    root().unwrap_or_else(|_| {
        log::error!("Failed to initialize filesystem");
        FSObjRef::empty_compound() // Return an empty filesystem on error
    })
}
