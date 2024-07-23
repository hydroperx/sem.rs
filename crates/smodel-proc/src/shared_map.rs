use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::Hash;
use std::rc::Rc;

#[derive(Clone)]
pub struct SharedMap<K, V>(Rc<RefCell<HashMap<K, V>>>);

impl<K, V> PartialEq for SharedMap<K, V> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl<K, V> Eq for SharedMap<K, V> {}

impl<K, V> Hash for SharedMap<K, V> {
    /// Performs hashing of the map by reference.
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.as_ptr().hash(state)
    }
}

impl<K, V> SharedMap<K, V> {
    pub fn new() -> Self {
        Self(Rc::new(RefCell::new(HashMap::new())))
    }

    pub fn get(&self, key: &K) -> Option<V> where K: Eq + Hash, V: Clone {
        self.0.borrow().get(key).map(|v| v.clone())
    }

    pub fn set(&mut self, key: K, value: V) where K: Eq + Hash {
        self.0.borrow_mut().insert(key, value);
    }

    pub fn _remove(&mut self, key: &K) -> Option<V> where K: Eq + Hash {
        self.0.borrow_mut().remove(key)
    }

    pub fn has(&self, key: &K) -> bool where K: Eq + Hash {
        self.0.borrow().contains_key(key)
    }

    pub fn _length(&self) -> usize {
        self.0.borrow().len()
    }

    pub fn _clone_content(&self) -> Self where K: Clone + Eq + Hash, V: Clone {
        let mut r = Self::new();
        for (k, v) in self.borrow().iter() {
            r.set(k.clone(), v.clone());
        }
        r
    }

    pub fn borrow(&self) -> std::cell::Ref<HashMap<K, V>> {
        self.0.borrow()
    }
}

impl<const N: usize, K: Eq + Hash, V> From<[(K, V); N]> for SharedMap<K, V> {
    fn from(value: [(K, V); N]) -> Self {
        Self::from_iter(value)
    }
}

impl<K: Eq + Hash, V> From<Vec<(K, V)>> for SharedMap<K, V> {
    fn from(value: Vec<(K, V)>) -> Self {
        Self::from_iter(value)
    }
}

impl<K: Eq + Hash, V> From<HashMap<K, V>> for SharedMap<K, V> {
    fn from(value: HashMap<K, V>) -> Self {
        Self::from_iter(value)
    }
}

impl<K: Eq + Hash, V> FromIterator<(K, V)> for SharedMap<K, V> {
    fn from_iter<T2: IntoIterator<Item = (K, V)>>(iter: T2) -> Self {
        let mut r = Self::new();
        for (k, v) in iter {
            r.set(k, v);
        }
        r
    }
}

impl<'a, K: Eq + Hash + Clone, V: Clone> FromIterator<(&'a K, &'a V)> for SharedMap<K, V> {
    fn from_iter<T2: IntoIterator<Item = (&'a K, &'a V)>>(iter: T2) -> Self {
        let mut r = Self::new();
        for (k, v) in iter {
            r.set(k.clone(), v.clone());
        }
        r
    }
}

impl<K, V> Extend<(K, V)> for SharedMap<K, V> where K: Eq + Hash {
    fn extend<T: IntoIterator<Item = (K, V)>>(&mut self, iter: T) {
        for (k, v) in iter.into_iter() {
            self.set(k, v);
        }
    }
}

macro_rules! shared_map {
    ($($key:expr => $value:expr),*) => {
        SharedMap::from([$(($key, $value)),*])
    };
    ($($key:expr => $value:expr),+ ,) => {
        SharedMap::from([$(($key, $value)),+])
    };
}
