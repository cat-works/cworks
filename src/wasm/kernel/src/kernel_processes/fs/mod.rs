mod fs_obj;

use std::{collections::HashMap, rc::Rc, sync::Arc};

pub use fs_obj::FSObj;

impl<T: Into<FSObj>> From<Option<T>> for FSObj {
    fn from(value: Option<T>) -> Self {
        let mut map = HashMap::new();
        match value {
            Some(x) => {
                map.insert("has_data".to_string(), FSObj::Boolean(Arc::new(true)));
                map.insert("data".to_string(), x.into());
            }
            None => {
                map.insert("has_data".to_string(), FSObj::Boolean(Arc::new(false)));
            }
        }
        FSObj::Dict(Arc::new(map.into()))
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
