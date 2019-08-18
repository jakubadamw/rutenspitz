#![feature(map_get_key_value)]

#[macro_use]
extern crate arbitrary_model_tests;
#[macro_use]
extern crate derive_arbitrary;
#[macro_use]
extern crate honggfuzz;

use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{BuildHasher, Hash, Hasher};

pub struct BuildTrulyAwfulHasher {
    seed: u8,
}

impl BuildTrulyAwfulHasher {
    pub fn new(seed: u8) -> Self {
        Self { seed }
    }
}

impl BuildHasher for BuildTrulyAwfulHasher {
    type Hasher = TrulyAwfulHasher;

    fn build_hasher(&self) -> Self::Hasher {
        TrulyAwfulHasher::new(self.seed)
    }
}

pub struct TrulyAwfulHasher {
    hash_value: u8,
}

impl TrulyAwfulHasher {
    fn new(seed: u8) -> Self {
        Self { hash_value: seed }
    }
}

impl Hasher for TrulyAwfulHasher {
    fn write(&mut self, bytes: &[u8]) {
        if let Some(byte) = bytes.first() {
            self.hash_value = self.hash_value.wrapping_add(*byte) % 8;
        }
    }

    fn finish(&self) -> u64 {
        u64::from(self.hash_value)
    }
}
pub struct ModelHashMap<K, V>
where
    K: Eq + Hash,
{
    data: Vec<(K, V)>,
}

impl<K, V> ModelHashMap<K, V>
where
    K: Eq + Hash,
{
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    pub fn clear(&mut self) {
        self.data.clear()
    }

    pub fn contains_key(&self, k: &K) -> bool {
        self.data.iter().find(|probe| probe.0 == *k).is_some()
    }

    pub fn get(&self, k: &K) -> Option<&V> {
        self.data.iter().find(|probe| probe.0 == *k).map(|e| &e.1)
    }

    pub fn get_key_value(&self, k: &K) -> Option<(&K, &V)> {
        self.data
            .iter()
            .find(|probe| probe.0 == *k)
            .map(|e| (&e.0, &e.1))
    }

    pub fn get_mut(&mut self, k: &K) -> Option<&mut V> {
        self.data
            .iter_mut()
            .find(|probe| probe.0 == *k)
            .map(|e| &mut e.1)
    }

    pub fn insert(&mut self, k: K, v: V) -> Option<V> {
        if let Some(e) = self.data.iter_mut().find(|probe| probe.0 == k) {
            Some(std::mem::replace(&mut e.1, v))
        } else {
            self.data.push((k, v));
            None
        }
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn remove(&mut self, k: &K) -> Option<V> {
        let pos = self.data.iter().position(|probe| probe.0 == *k);
        pos.map(|idx| self.data.swap_remove(idx).1)
    }

    pub fn drain(&mut self) -> impl Iterator<Item = (K, V)> + '_ {
        self.data.drain(..)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.data.iter().map(|e| (&e.0, &e.1))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&K, &mut V)> {
        self.data.iter_mut().map(|e| (&e.0, &mut e.1))
    }

    pub fn keys(&self) -> impl Iterator<Item = &K> {
        self.data.iter().map(|e| &e.0)
    }

    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.data.iter().map(|e| &e.1)
    }

    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut V> {
        self.data.iter_mut().map(|e| &mut e.1)
    }
}

fn sort_iterator<T: Ord, I: Iterator<Item = T>>(i: I) -> Vec<T> {
    let mut v: Vec<_> = i.collect::<Vec<_>>();
    v.sort();
    v
}

arbitrary_stateful_operations! {
    model = ModelHashMap<K, V>,
    tested = HashMap<K, V, BuildTrulyAwfulHasher>,

    type_parameters = <
        K: Clone + Debug + Eq + Hash + Ord,
        V: Clone + Debug + Eq + Ord
    >,

    methods {
        equal {
            fn clear(&mut self);
            fn contains_key(&self, k: &K) -> bool;
            fn get(&self, k: &K) -> Option<&V>;
            fn get_key_value(&self, k: &K) -> Option<(&K, &V)>;
            fn get_mut(&mut self, k: &K) -> Option<&mut V>;
            fn insert(&mut self, k: K, v: V) -> Option<V>;
            // Tested as invariants, so no longer needed.
            // fn is_empty(&self) -> bool;
            // fn len(&self) -> usize;
            fn remove(&mut self, k: &K) -> Option<V>;
        }

        equal_with(sort_iterator) {
            fn drain(&mut self) -> impl Iterator<Item = (K, V)>;
            fn iter(&self) -> impl Iterator<Item = (&K, &V)>;
            fn iter_mut(&self) -> impl Iterator<Item = (&K, &mut V)>;
            fn keys(&self) -> impl Iterator<Item = &K>;
            fn values(&self) -> impl Iterator<Item = &V>;
            fn values_mut(&mut self) -> impl Iterator<Item = &mut V>;
        }
    }

    pre {
        let prev_capacity = tested.capacity();
    }

    post {
        // A bit of a hack.
        if &self == &Self::clear {
            assert_eq!(tested.capacity(), prev_capacity);
        }

        assert!(tested.capacity() >= model.len());
        assert_eq!(tested.is_empty(), model.is_empty());
        assert_eq!(tested.len(), model.len());
    }
}

const MAX_RING_SIZE: usize = 16_384;
//const MAX_RING_SIZE: usize = 65_536;

fn fuzz_cycle(data: &[u8]) -> Result<(), ()> {
    use arbitrary::{Arbitrary, FiniteBuffer};

    let mut ring = FiniteBuffer::new(&data, MAX_RING_SIZE).map_err(|_| ())?;
    let hash_seed: u8 = Arbitrary::arbitrary(&mut ring)?;
    let capacity: u8 = Arbitrary::arbitrary(&mut ring)?;

    let mut model = ModelHashMap::<u16, u16>::new();
    let mut tested: HashMap<u16, u16, BuildTrulyAwfulHasher> =
        HashMap::with_capacity_and_hasher(capacity as usize, BuildTrulyAwfulHasher::new(hash_seed));

    let mut _op_trace = String::new();
    while let Ok(op) = <op::Op<u16, u16> as Arbitrary>::arbitrary(&mut ring) {
        #[cfg(fuzzing_debug)]
        _op_trace.push_str(format!("{}\n", op.to_string()));
        op.execute_and_compare(&mut model, &mut tested);
    }

    Ok(())
}

fn main() -> Result<(), ()> {
    better_panic::install();

    loop {
        fuzz!(|data: &[u8]| {
            let _ = fuzz_cycle(data);
        });
    }
}
