use std::{
    any::{Any, TypeId},
    collections::{HashMap, hash_map::Entry},
};

pub struct AnyMap {
    map: HashMap<TypeId, Box<dyn Any>>,
}

impl AnyMap {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn values(&self) -> impl Iterator<Item = &Box<dyn Any>> {
        self.map.values()
    }

    pub fn insert<T: Any>(&mut self, value: T) -> Option<T> {
        match self.map.entry(TypeId::of::<T>()) {
            Entry::Occupied(mut e) => {
                let old_value = e.insert(Box::new(value)).downcast::<T>().ok()?;
                Some(*old_value)
            }
            Entry::Vacant(e) => {
                e.insert(Box::new(value));
                None
            }
        }
    }

    pub fn get<T: Any>(&self) -> Option<&T> {
        self.map.get(&TypeId::of::<T>())?.downcast_ref::<T>()
    }

    pub fn get_mut<T: Any>(&mut self) -> Option<&mut T> {
        self.map.get_mut(&TypeId::of::<T>())?.downcast_mut::<T>()
    }
}

impl Default for AnyMap {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_map() {
        let mut map = AnyMap::new();

        assert!(map.insert("bla").is_none());
        assert!(map.insert(1234_u32).is_none());

        assert_eq!(*map.get::<u32>().unwrap(), 1234);
        assert_eq!(*map.get::<&str>().unwrap(), "bla");

        *map.get_mut::<u32>().unwrap() += 1;
        assert_eq!(*map.get::<u32>().unwrap(), 1235);

        // return old value on replace
        assert_eq!(map.insert(1111_u32).unwrap(), 1235);
    }
}
