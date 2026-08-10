mod object;
pub use object::FileStat;
pub use object::Object;

mod intrinsic_fs_obj;
pub use intrinsic_fs_obj::IntrinsicFSObj;

mod fs_obj_ref;
pub use fs_obj_ref::FSObjRef;

mod compound_fs_obj;
pub use compound_fs_obj::CompoundFSObj;
