#![feature(map_get_key_value)]

#[macro_use]
extern crate arbitrary_model_tests;
#[macro_use]
extern crate derive_arbitrary;
#[macro_use]
extern crate honggfuzz;

use linked_hash_map::LinkedHashMap;

use std::fmt::Debug;
use std::hash::Hash;

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

    pub fn get_mut(&mut self, k: &K) -> Option<&mut V> {
        self.data
            .iter_mut()
            .find(|probe| probe.0 == *k)
            .map(|e| &mut e.1)
    }

    pub fn insert(&mut self, k: K, v: V) -> Option<V> {
        let old_value = self.remove(&k);
        self.data.push((k, v));
        old_value
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn remove(&mut self, k: &K) -> Option<V> {
        let pos = self.data.iter().position(|probe| probe.0 == *k);
        pos.map(|idx| {
            let mut rest = self.data.split_off(idx);
            let mut it = rest.drain(..);
            let el = it.next().unwrap().1;
            self.data.extend(it);
            el
        })
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
}

fn collect_iterator<T, I: Iterator<Item = T>>(i: I) -> Vec<T> {
    i.collect()
}

arbitrary_stateful_operations! {
    model = ModelHashMap<K, V>,
    tested = LinkedHashMap<K, V>,

    type_parameters = <
        K: Clone + Debug + Eq + Hash + Ord,
        V: Clone + Debug + Eq + Ord
    >,

    methods {
        equal {
            fn clear(&mut self);
            fn contains_key(&self, k: &K) -> bool;
            fn get(&self, k: &K) -> Option<&V>;
            fn get_mut(&mut self, k: &K) -> Option<&mut V>;
            fn insert(&mut self, k: K, v: V) -> Option<V>;
            fn is_empty(&self) -> bool;
            fn len(&self) -> usize;
            fn remove(&mut self, k: &K) -> Option<V>;
        }

        equal_with(collect_iterator) {
            fn iter(&self) -> impl Iterator<Item = (&K, &V)>;
            fn iter_mut(&self) -> impl Iterator<Item = (&K, &mut V)>;
            fn keys(&self) -> impl Iterator<Item = &K>;
            fn values(&self) -> impl Iterator<Item = &V>;
        }
    }
}

const MAX_RING_SIZE: usize = 65_536;

fn fuzz_cycle(data: &[u8]) -> Result<(), ()> {
    use arbitrary::{Arbitrary, FiniteBuffer};

    let mut ring = FiniteBuffer::new(&data, MAX_RING_SIZE).map_err(|_| ())?;
    let capacity: u8 = Arbitrary::arbitrary(&mut ring)?;

    let mut model = ModelHashMap::<u16, u16>::new();
    let mut tested = LinkedHashMap::<u16, u16>::with_capacity(capacity as usize);

    let mut _op_trace = String::new();
    while let Ok(op) = <op::Op<u16, u16> as Arbitrary>::arbitrary(&mut ring) {
        #[cfg(fuzzing_debug)] _op_trace.push_str(format!("{}\n", op.to_string()));
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
