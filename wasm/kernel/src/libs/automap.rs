use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};

pub struct AutoMap<T> {
    map: HashMap<u64, T>,
    next_id: u64,
}

impl<T> AutoMap<T> {
    pub fn new(min_id: u64) -> Self {
        Self {
            map: HashMap::new(),
            next_id: min_id,
        }
    }
    fn find_free_id(&mut self) -> u64 {
        let mut i = self.next_id;

        while self.map.contains_key(&i) {
            i += 1;
        }
        self.next_id = i + 1;

        i
    }
    pub fn add_value(&mut self, value: T) -> u64 {
        let i = self.find_free_id();

        self.map.insert(i, value);

        i
    }
}

impl<T> Deref for AutoMap<T> {
    type Target = HashMap<u64, T>;

    fn deref(&self) -> &Self::Target {
        &self.map
    }
}
impl<T> DerefMut for AutoMap<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.map
    }
}
