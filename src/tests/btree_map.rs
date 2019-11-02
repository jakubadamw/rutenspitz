#![allow(clippy::find_map, clippy::filter_map, clippy::must_use_candidate)]

#[macro_use]
extern crate derive_arbitrary;

use arbitrary_model_tests::arbitrary_stateful_operations;
use honggfuzz::fuzz;

use std::collections::BTreeMap;
use std::fmt::Debug;

#[derive(Default)]
pub struct ModelBTreeMap<K, V>
where
    K: Eq + Ord,
{
    data: Vec<(K, V)>,
}

impl<K, V> ModelBTreeMap<K, V>
where
    K: Eq + Ord,
{
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

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

    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.data.iter().map(|e| (&e.0, &e.1))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&K, &mut V)> {
        self.data.iter_mut().map(|e| (&e.0, &mut e.1))
    }

    pub fn keys(&self) -> impl Iterator<Item = &K> {
        self.data.iter().map(|e| &e.0)
    }

    pub fn range(&mut self, range: std::ops::Range<K>) -> impl Iterator<Item = (&K, &V)> {
        self.range_mut(range).map(|e| (&*e.0, &*e.1))
    }

    pub fn range_mut(&mut self, range: std::ops::Range<K>) -> impl Iterator<Item = (&K, &mut V)> {
        self.data
            .iter_mut()
            .filter(move |e| e.0 >= range.start && e.0 < range.end)
            .map(|e| (&e.0, &mut e.1))
    }

    pub fn split_off(&mut self, key: &K) -> impl IntoIterator<Item = (K, V)> {
        let (a, b) = self.data.drain(..).partition(|probe| probe.0 < *key);
        self.data = a;
        b
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

fn sort_iterable<T: Ord, I: IntoIterator<Item = T>>(i: I) -> Vec<T> {
    let mut v: Vec<_> = i.into_iter().collect::<Vec<_>>();
    v.sort();
    v
}

arbitrary_stateful_operations! {
    model = ModelBTreeMap<K, V>,
    tested = BTreeMap<K, V>,

    type_parameters = <
        K: Clone + Debug + Eq + Ord,
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
            fn is_empty(&self) -> bool;
            fn len(&self) -> usize;
            fn remove(&mut self, k: &K) -> Option<V>;
        }

        equal_with(sort_iterator) {
            fn iter(&self) -> impl Iterator<Item = (&K, &V)>;
            fn iter_mut(&self) -> impl Iterator<Item = (&K, &mut V)>;
            fn keys(&self) -> impl Iterator<Item = &K>;
            fn range(&self, range: std::ops::Range<K>) -> impl Iterator<Item = (&K, &V)>;
            fn range_mut(&self, range: std::ops::Range<K>) -> impl Iterator<Item = (&K, &mut V)>;
            fn values(&self) -> impl Iterator<Item = &V>;
            fn values_mut(&mut self) -> impl Iterator<Item = &mut V>;
        }

        equal_with(sort_iterable) {
            fn split_off(&mut self, k: &K) -> impl IntoIterator<Item = (&K, &V)>;
        }
    }
}

const MAX_RING_SIZE: usize = 65_536;

fn fuzz_cycle(data: &[u8]) -> Result<(), ()> {
    use arbitrary::{Arbitrary, FiniteBuffer};

    let mut ring = FiniteBuffer::new(&data, MAX_RING_SIZE).map_err(|_| ())?;
    let mut model = ModelBTreeMap::<u16, u16>::new();
    let mut tested = BTreeMap::<u16, u16>::new();

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
