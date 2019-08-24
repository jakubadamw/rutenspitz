#![allow(clippy::find_map)]
#![feature(map_get_key_value)]

#[macro_use]
extern crate arbitrary_model_tests;
#[macro_use]
extern crate derive_arbitrary;
#[macro_use]
extern crate honggfuzz;

use indexmap::IndexMap;

use std::fmt::Debug;
use std::hash::Hash;

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

    pub fn get_full(&self, k: &K) -> Option<(usize, &K, &V)> {
        self.data
            .iter()
            .enumerate()
            .find(|(_, probe)| probe.0 == *k)
            .map(|(i, e)| (i, &e.0, &e.1))
    }

    pub fn get_full_mut(&mut self, k: &K) -> Option<(usize, &K, &mut V)> {
        self.data
            .iter_mut()
            .enumerate()
            .find(|(_, probe)| probe.0 == *k)
            .map(|(i, e)| (i, &e.0, &mut e.1))
    }

    pub fn get_index(&self, index: usize) -> Option<(&K, &V)> {
        self.data.get(index).map(|el| (&el.0, &el.1))
    }

    pub fn get_index_mut(&mut self, index: usize) -> Option<(&mut K, &mut V)> {
        self.data.get_mut(index).map(|el| (&mut el.0, &mut el.1))
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

    pub fn pop(&mut self) -> Option<(K, V)> {
        self.data.pop()
    }

    pub fn swap_remove(&mut self, key: &K) -> Option<(V)> {
        self.swap_remove_full(key).map(|(_, _, v)| v)
    }

    pub fn swap_remove_full(&mut self, key: &K) -> Option<(usize, K, V)> {
        let pos = self.data.iter().position(|probe| probe.0 == *key);
        pos.map(|idx| (idx, self.data.swap_remove(idx)))
            .map(|(idx, (k, v))| (idx, k, v))
    }

    pub fn swap_remove_index(&mut self, index: usize) -> Option<(K, V)> {
        if index >= self.data.len() {
            return None;
        }
        Some(self.data.swap_remove(index))
    }

    pub fn remove(&mut self, k: &K) -> Option<V> {
        self.swap_remove(k)
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
    tested = IndexMap<K, V>,

    type_parameters = <
        K: Clone + Copy + Debug + Eq + Hash + Ord,
        V: Clone + Debug + Eq + Ord
    >,

    methods {
        equal {
            fn clear(&mut self);
            fn contains_key(&self, k: &K) -> bool;
            fn get(&self, k: &K) -> Option<&V>;
            fn get_full(&self, k: &K) -> Option<(usize, &K, &V)>;
            fn get_full_mut(&mut self, k: &K) -> Option<(usize, &K, &mut V)>;
            fn get_index(&self, index: usize) -> Option<(&K, &V)>;
            fn get_index_mut(&mut self, index: usize) -> Option<(&mut K, &mut V)>;
            fn get_mut(&mut self, k: &K) -> Option<&mut V>;
            fn insert(&mut self, k: K, v: V) -> Option<V>;
            fn is_empty(&self) -> bool;
            fn len(&self) -> usize;
            fn pop(&mut self) -> Option<(K, V)>;
            fn remove(&mut self, k: &K) -> Option<V>;
            fn swap_remove(&mut self, key: &K) -> Option<V>;
            fn swap_remove_full(&mut self, key: &K) -> Option<(usize, K, V)>;
            fn swap_remove_index(&mut self, index: usize) -> Option<(K, V)>;
        }

        equal_with(sort_iterator) {
            fn iter(&self) -> impl Iterator<Item = (&K, &V)>;
            fn iter_mut(&self) -> impl Iterator<Item = (&K, &mut V)>;
            fn keys(&self) -> impl Iterator<Item = &K>;
            fn values(&self) -> impl Iterator<Item = &V>;
            fn values_mut(&mut self) -> impl Iterator<Item = &mut V>;
        }
    }
}

const MAX_RING_SIZE: usize = 65_536;

fn fuzz_cycle(data: &[u8]) -> Result<(), ()> {
    use arbitrary::{Arbitrary, FiniteBuffer};

    let mut ring = FiniteBuffer::new(&data, MAX_RING_SIZE).map_err(|_| ())?;
    let capacity: u8 = Arbitrary::arbitrary(&mut ring)?;

    let mut model = ModelHashMap::<u16, u16>::default();
    let mut tested = IndexMap::<u16, u16>::with_capacity(capacity as usize);

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
