#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::find_map)]
#![feature(shrink_to)]

#[macro_use]
extern crate arbitrary_model_tests;
#[macro_use]
extern crate derive_arbitrary;
#[macro_use]
extern crate honggfuzz;

use hashbrown::HashMap;

use std::fmt::Debug;
use std::hash::{BuildHasher, Hash};

pub struct BuildAHasher {
    seed: u128,
}

impl BuildAHasher {
    pub fn new(seed: u128) -> Self {
        Self { seed }
    }
}

impl BuildHasher for BuildAHasher {
    type Hasher = ahash::AHasher;

    fn build_hasher(&self) -> Self::Hasher {
        ahash::AHasher::new_with_keys((self.seed >> 64) as u64, self.seed as u64)
    }
}

#[derive(Default)]
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
    pub fn clear(&mut self) {
        self.data.clear()
    }

    pub fn contains_key(&self, k: &K) -> bool {
        self.data.iter().any(|probe| probe.0 == *k)
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

    pub fn shrink_to(&mut self, min_capacity: usize) {
        self.data.shrink_to(std::cmp::min(self.data.capacity(), std::cmp::max(min_capacity, self.data.len())));
    }

    pub fn shrink_to_fit(&mut self) {
        self.data.shrink_to_fit();
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
    tested = HashMap<K, V, BuildAHasher>,

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
            fn remove(&mut self, k: &K) -> Option<V>;
            fn shrink_to(&mut self, min_capacity: usize);
            fn shrink_to_fit(&mut self);
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
        if self == Self::clear {
            assert_eq!(tested.capacity(), prev_capacity,
                "capacity: {}, previous: {}",
                tested.capacity(), prev_capacity);
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

    let capacity: usize = Arbitrary::arbitrary(&mut ring)?;
    let hash_seed: u128 = Arbitrary::arbitrary(&mut ring)?;
    
    let mut model = ModelHashMap::<u16, u16>::default();
    let mut tested: HashMap<u16, u16, BuildAHasher> =
        HashMap::with_capacity_and_hasher(capacity as usize, BuildAHasher::new(hash_seed));

    let mut _op_trace = String::new();
    while let Ok(op) = <op::Op<u16, u16> as Arbitrary>::arbitrary(&mut ring) {
        #[cfg(fuzzing_debug)]
        _op_trace.push_str(&format!("{}\n", op.to_string()));
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
