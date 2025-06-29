use std::{
    any::TypeId,
    collections::{HashMap, hash_map::Entry},
    fmt::Debug,
    hash::Hash,
};

pub struct ObjectMap<Key: Hash + Eq + Debug, Value> {
    map: HashMap<TypeId, HashMap<Key, Value>>,
}

impl<Key: Hash + Eq + Debug, Value> ObjectMap<Key, Value> {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn insert<T: 'static>(&mut self, key: Key, value: Value) -> Option<Value> {
        let old_value = match self.map.entry(TypeId::of::<T>()) {
            Entry::Occupied(mut e) => e.get_mut().insert(key, value)?,
            Entry::Vacant(e) => e.insert(HashMap::new()).insert(key, value)?,
        };

        Some(old_value)
    }

    pub fn remove<T: 'static>(&mut self, key: &Key) -> Option<Value> {
        if let Entry::Occupied(mut e) = self.map.entry(TypeId::of::<T>()) {
            Some(e.get_mut().remove(key)?)
        } else {
            tracing::warn!(target: "core-utils", "Missing entry: {:?}", key);
            None
        }
    }

    pub fn get<T: 'static>(&self, key: &Key) -> Option<&Value> {
        self.map.get(&TypeId::of::<T>())?.get(key)
    }

    pub fn get_mut<T: 'static>(&mut self, key: &Key) -> Option<&mut Value> {
        self.map.get_mut(&TypeId::of::<T>())?.get_mut(key)
    }
}

impl<Key: Hash + Eq + Debug, Value> Default for ObjectMap<Key, Value> {
    fn default() -> Self {
        Self::new()
    }
}
