use std::{hash::{Hash, Hasher}, collections::{HashSet, hash_set::Iter, hash_map::DefaultHasher}};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HashableSet<T: Hash + Eq> {
    pub set: HashSet<T>,
    pub hash: u64,
}

impl<T: Hash + Eq> HashableSet<T> {
    pub fn new() -> HashableSet<T> {
        HashableSet {
            set: HashSet::new(),
            hash: 0,
        }
    }

    pub fn insert(&mut self, value: T) -> bool {
        let mut s = DefaultHasher::new();
        value.hash(&mut s);
        self.hash = self.hash ^ s.finish();
        let result = self.set.insert(value);
        if !result {
            self.hash = self.hash ^ s.finish();
        }
        result
    }

    pub fn iter(&self) -> Iter<T> {
        self.set.iter()
    }

    pub fn len(&self) -> usize {
        self.set.len()
    }

    pub fn contains(&self, value: &T) -> bool {
        self.set.contains(value)
    }
}

impl<T: Hash + Eq> Hash for HashableSet<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hash.hash(state);
    }
}

impl<T: Hash + Eq> FromIterator<T> for HashableSet<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut set = HashableSet::new();
        for value in iter {
            set.insert(value);
        }
        set
    }
}
