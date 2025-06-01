use std::{cell::RefCell, fmt::Debug, ops::Deref, rc::Rc};

use super::FSObj;

pub struct FSObjRef(Rc<RefCell<Box<dyn FSObj>>>);

impl<T: FSObj + 'static> From<T> for FSObjRef {
    fn from(obj: T) -> Self {
        FSObjRef(Rc::new(RefCell::new(Box::new(obj))))
    }
}

impl Debug for FSObjRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FSObjRef({:?})", self.0)
    }
}

impl Clone for FSObjRef {
    fn clone(&self) -> Self {
        FSObjRef(self.0.clone())
    }
}

impl Deref for FSObjRef {
    type Target = Rc<RefCell<Box<dyn FSObj>>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
