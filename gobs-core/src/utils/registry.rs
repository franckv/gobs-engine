use std::{
    any::{Any, TypeId},
    collections::{HashMap, hash_map::Entry},
};

use uuid::Uuid;

type Key = Uuid;

pub struct ObjectRegistry {
    registry: HashMap<TypeId, HashMap<Key, Box<dyn Any>>>,
}

impl ObjectRegistry {
    pub fn new() -> Self {
        Self {
            registry: HashMap::new(),
        }
    }

    pub fn insert<T: Any>(&mut self, key: Key, value: T) -> Option<T> {
        let old_value = match self.registry.entry(TypeId::of::<T>()) {
            Entry::Occupied(mut e) => e
                .get_mut()
                .insert(key, Box::new(value))?
                .downcast::<T>()
                .ok()?,
            Entry::Vacant(e) => e
                .insert(HashMap::new())
                .insert(key, Box::new(value))?
                .downcast::<T>()
                .ok()?,
        };

        Some(*old_value)
    }

    pub fn get<T: Any>(&self, key: &Key) -> Option<&T> {
        self.registry
            .get(&TypeId::of::<T>())?
            .get(key)?
            .downcast_ref::<T>()
    }

    pub fn get_mut<T: Any>(&mut self, key: &Key) -> Option<&mut T> {
        self.registry
            .get_mut(&TypeId::of::<T>())?
            .get_mut(key)?
            .downcast_mut::<T>()
    }
}

impl Default for ObjectRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_registry() {
        let mut registry = ObjectRegistry::new();

        let key1 = Uuid::new_v4();
        let key2 = Uuid::new_v4();

        assert!(registry.insert(key1, "bla").is_none());
        assert!(registry.insert(key1, 1234_u32).is_none());
        assert!(registry.insert(key2, 4321_u32).is_none());
        assert!(registry.insert(key2, "bli").is_none());
        assert!(registry.insert(key2, vec![1, 2, 3, 4]).is_none());

        assert_eq!(*registry.get::<u32>(&key1).unwrap(), 1234);
        assert_eq!(*registry.get::<&str>(&key1).unwrap(), "bla");
        assert_eq!(*registry.get::<u32>(&key2).unwrap(), 4321);
        assert_eq!(*registry.get::<&str>(&key2).unwrap(), "bli");
        assert_eq!(registry.get::<Vec<i32>>(&key2).unwrap().len(), 4);

        *registry.get_mut::<u32>(&key1).unwrap() += 1;
        assert_eq!(*registry.get::<u32>(&key1).unwrap(), 1235);

        // return old value on replace
        assert_eq!(registry.insert(key1, 1111_u32).unwrap(), 1235);
    }
}
