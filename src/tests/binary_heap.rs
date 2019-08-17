#[macro_use] extern crate arbitrary_model_tests;
#[macro_use] extern crate derive_arbitrary;
#[macro_use] extern crate honggfuzz;

use std::collections::BinaryHeap;
use std::fmt::Debug;

pub struct ModelBinaryHeap<T>
where
    T: Ord
{
    data: Vec<T>,
}

impl<T> ModelBinaryHeap<T>
where
    T: Ord,
{
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    pub fn clear(&mut self) {
        self.data.clear()
    }
   
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
    
    pub fn len(&self) -> usize {
        self.data.len()
    }
    
    pub fn peek(&self) -> Option<&T> {
        self.data.iter().max()
    }

    pub fn pop(&mut self) -> Option<T> {
        let max = self.data.iter().enumerate().max_by_key(|probe| probe.1).map(|probe| probe.0);
        max.map(|idx| self.data.swap_remove(idx))
    }

    pub fn push(&mut self, item: T) {
        self.data.push(item)
    }

    pub fn drain(&mut self) -> impl Iterator<Item = T> + '_ {
        self.data.drain(..)
    }
}

fn sort_iterator<T: Ord, I: Iterator<Item = T>>(i: I) -> Vec<T> {
    let mut v: Vec<_> = i.collect::<Vec<_>>();
    v.sort();
    v
}

arbitrary_stateful_operations! {
    model = ModelBinaryHeap<T>,
    tested = BinaryHeap<T>,
    
    type_parameters = <
        T: Clone + Debug + Eq + Ord
    >,

    methods {
        equal {
            fn clear(&mut self);
            fn is_empty(&self) -> bool;
            fn len(&self) -> usize;
            fn peek(&self) -> Option<&T>;
            fn push(&mut self, item: T);
            /*fn contains_key(&self, k: &K) -> bool;
            fn get(&self, k: &K) -> Option<&V>;
            fn get_key_value(&self, k: &K) -> Option<(&K, &V)>;
            fn get_mut(&mut self, k: &K) -> Option<&mut V>;
            fn insert(&mut self, k: K, v: V) -> Option<V>;
            fn remove(&mut self, k: &K) -> Option<V>;*/
        }

        equal_with(sort_iterator) {
            fn drain(&mut self) -> impl Iterator<Item = T>;
            /*fn iter(&self) -> impl Iterator<Item = (&K, &V)>;
            fn iter_mut(&self) -> impl Iterator<Item = (&K, &mut V)>;
            fn keys(&self) -> impl Iterator<Item = &K>;
            fn values(&self) -> impl Iterator<Item = &V>;
            fn values_mut(&mut self) -> impl Iterator<Item = &mut V>;*/
        }
    }
}

const MAX_RING_SIZE: usize = 16_384;

fn fuzz_cycle(data: &[u8]) -> Result<(), ()> {
    use arbitrary::{Arbitrary, FiniteBuffer};
    
    let mut ring = FiniteBuffer::new(&data, MAX_RING_SIZE)
        .map_err(|_| ())?;
    let capacity: u8 = Arbitrary::arbitrary(&mut ring)?;
    
    let mut model = ModelBinaryHeap::<u16>::new();
    let mut tested = BinaryHeap::<u16>::with_capacity(capacity as usize);

    let mut op_trace = vec![];
    while let Ok(op) = <op::Op<u16> as Arbitrary>::arbitrary(&mut ring) {
        op_trace.push(op.clone());
        op.execute(&mut model, &mut tested);
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

