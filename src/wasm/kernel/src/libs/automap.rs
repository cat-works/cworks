use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};

pub struct AutoMap<T>(HashMap<u128, T>);

impl<T> AutoMap<T> {
    pub fn new() -> AutoMap<T> {
        AutoMap(HashMap::new())
    }
    fn find_free_id(&self) -> u128 {
        let mut i: u128 = 0;

        while self.contains_key(&i) {
            i += 1;
        }

        i
    }
    pub fn add_value(&mut self, value: T) -> u128 {
        let i = self.find_free_id();

        self.insert(i, value);

        i
    }
}
impl<T> Deref for AutoMap<T> {
    type Target = HashMap<u128, T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<T> DerefMut for AutoMap<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
