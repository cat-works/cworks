use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};

pub struct AutoMap<T> {
    map: HashMap<u128, T>,
    begin: u128,
}

impl<T> AutoMap<T> {
    pub fn new(begin: u128) -> Self {
        Self {
            map: HashMap::new(),
            begin,
        }
    }
    fn find_free_id(&self) -> u128 {
        let mut i: u128 = self.begin;

        while self.map.contains_key(&i) {
            i += 1;
        }

        i
    }
    pub fn add_value(&mut self, value: T) -> u128 {
        let i = self.find_free_id();

        self.map.insert(i, value);

        i
    }
}
impl<T> Deref for AutoMap<T> {
    type Target = HashMap<u128, T>;

    fn deref(&self) -> &Self::Target {
        &self.map
    }
}
impl<T> DerefMut for AutoMap<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.map
    }
}
