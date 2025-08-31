use std::{
    any::{Any, TypeId},
    collections::{HashMap, hash_map::Entry},
};

use slotmap::{DefaultKey, SlotMap};

pub type Key = DefaultKey;

pub struct ObjectArena {
    registry: HashMap<TypeId, SlotMap<Key, Box<dyn Any>>>,
}

impl ObjectArena {
    pub fn new() -> Self {
        Self {
            registry: HashMap::new(),
        }
    }

    pub fn values<T: Any>(&self) -> impl Iterator<Item = &T> {
        self.registry
            .get(&TypeId::of::<T>())
            .into_iter()
            .flat_map(|entries| entries.values().flat_map(|entry| entry.downcast_ref::<T>()))
    }

    pub fn values_mut<T: Any>(&mut self) -> impl Iterator<Item = &mut T> {
        self.registry
            .get_mut(&TypeId::of::<T>())
            .into_iter()
            .flat_map(|entries| {
                entries
                    .values_mut()
                    .flat_map(|entry| entry.downcast_mut::<T>())
            })
    }

    pub fn insert<T: Any>(&mut self, value: T) -> Key {
        match self.registry.entry(TypeId::of::<T>()) {
            Entry::Occupied(mut e) => e.get_mut().insert(Box::new(value)),
            Entry::Vacant(e) => e.insert(SlotMap::new()).insert(Box::new(value)),
        }
    }

    pub fn insert_with_key<F, T: Any>(&mut self, f: F) -> Key
    where
        F: FnOnce(Key) -> T,
    {
        match self.registry.entry(TypeId::of::<T>()) {
            Entry::Occupied(mut e) => e.get_mut().insert_with_key(|key| Box::new(f(key))),
            Entry::Vacant(e) => e
                .insert(SlotMap::new())
                .insert_with_key(|key| Box::new(f(key))),
        }
    }

    pub fn remove<T: Any>(&mut self, key: Key) -> Option<T> {
        if let Entry::Occupied(mut e) = self.registry.entry(TypeId::of::<T>()) {
            Some(*(e.get_mut().remove(key)?.downcast::<T>().ok()?))
        } else {
            tracing::warn!(target: "core-utils", "Missing entry: {:?}", key);
            None
        }
    }

    pub fn get<T: Any>(&self, key: Key) -> Option<&T> {
        self.registry
            .get(&TypeId::of::<T>())?
            .get(key)?
            .downcast_ref::<T>()
    }

    pub fn get_mut<T: Any>(&mut self, key: Key) -> Option<&mut T> {
        self.registry
            .get_mut(&TypeId::of::<T>())?
            .get_mut(key)?
            .downcast_mut::<T>()
    }
}

impl Default for ObjectArena {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_arena() {
        let mut arena = ObjectArena::new();

        let key1 = arena.insert("bla");
        let key2 = arena.insert(1234_u32);
        let key3 = arena.insert(vec![1, 2, 3, 4]);

        assert_eq!(*arena.get::<&str>(key1).unwrap(), "bla");
        assert_eq!(*arena.get::<u32>(key2).unwrap(), 1234);
        assert_eq!(arena.get::<Vec<i32>>(key3).unwrap().len(), 4);

        *arena.get_mut::<u32>(key2).unwrap() += 1;
        assert_eq!(*arena.get::<u32>(key2).unwrap(), 1235);
    }
}
