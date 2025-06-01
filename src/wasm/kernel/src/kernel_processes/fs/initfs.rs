use super::fs_obj::{CompoundFSObj, FSObjRef};

fn usr(root: FSObjRef) -> FSObjRef {
    let usr: FSObjRef = CompoundFSObj::with_parent(root).into();

    usr.borrow_mut().add_child(
        "mime".to_string(),
        CompoundFSObj::with_parent(usr.clone()).into(),
    );
    usr.borrow_mut().add_child(
        "app".to_string(),
        CompoundFSObj::with_parent(usr.clone()).into(),
    );
    usr.borrow_mut().add_child(
        "ref".to_string(),
        CompoundFSObj::with_parent(usr.clone()).into(),
    );

    usr
}

fn mnt(root: FSObjRef) -> FSObjRef {
    CompoundFSObj::with_parent(root).into()
}

fn workspace(root: FSObjRef) -> FSObjRef {
    CompoundFSObj::with_parent(root).into()
}

pub fn initfs() -> FSObjRef {
    let root: FSObjRef = CompoundFSObj::new().into();

    root.borrow_mut()
        .add_child("usr".to_string(), usr(root.clone()));
    root.borrow_mut()
        .add_child("mnt".to_string(), mnt(root.clone()));
    root.borrow_mut()
        .add_child("workspace".to_string(), workspace(root.clone()));

    root
}
