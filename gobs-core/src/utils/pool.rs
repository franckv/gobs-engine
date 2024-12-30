use std::{
    collections::{hash_map::Entry, HashMap},
    hash::Hash,
};

pub struct ObjectPool<K: Hash + Eq, V> {
    pool: HashMap<K, Vec<V>>,
}

impl<K: Hash + Eq, V> ObjectPool<K, V> {
    pub fn new() -> Self {
        Self {
            pool: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: K, value: V) {
        match self.pool.entry(key) {
            Entry::Occupied(mut e) => {
                e.get_mut().push(value);
            }
            Entry::Vacant(e) => {
                e.insert(vec![value]);
            }
        }
    }

    pub fn get(&self, key: &K) -> Option<&Vec<V>> {
        self.pool.get(key)
    }

    pub fn get_mut(&mut self, key: &K) -> Option<&mut Vec<V>> {
        self.pool.get_mut(key)
    }

    pub fn pop(&mut self, key: &K) -> Option<V> {
        self.pool.get_mut(key)?.pop()
    }

    pub fn contains(&self, key: &K) -> bool {
        self.pool.get(key).is_some_and(|v| !v.is_empty())
    }
}

impl<K: Hash + Eq, V> Default for ObjectPool<K, V> {
    fn default() -> Self {
        Self::new()
    }
}
