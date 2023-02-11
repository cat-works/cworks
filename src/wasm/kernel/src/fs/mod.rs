mod fs_cursor;
mod fs_obj;
mod ref_or_val;

use std::{collections::HashMap, rc::Rc};

// pub use fs_cursor::FSCursor;
pub use fs_obj::FSObj;
pub use ref_or_val::RefOrVal;

impl<T: Into<FSObj>> From<Option<T>> for FSObj {
    fn from(value: Option<T>) -> Self {
        let mut map = HashMap::new();
        match value {
            Some(x) => {
                map.insert("has_data".to_string(), FSObj::Boolean(RefOrVal::Val(true)));
                map.insert("data".to_string(), x.into());
            }
            None => {
                map.insert("has_data".to_string(), FSObj::Boolean(RefOrVal::Val(false)));
            }
        }
        FSObj::Dist(RefOrVal::Val(map))
    }
}
impl<T: Into<FSObj>> From<Box<T>> for FSObj {
    fn from(value: Box<T>) -> Self {
        (*value).into()
    }
}
impl<T: Into<FSObj>> From<Rc<T>> for FSObj {
    fn from(value: Rc<T>) -> Self {
        value.into()
    }
}
