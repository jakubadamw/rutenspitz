#![allow(clippy::must_use_candidate)]

use honggfuzz::fuzz;
use rutenspitz::arbitrary_stateful_operations;

use std::collections::BinaryHeap;
use std::fmt::Debug;

#[derive(Default)]
pub struct ModelBinaryHeap<T>
where
    T: Ord,
{
    data: Vec<T>,
}

impl<T> ModelBinaryHeap<T>
where
    T: Ord,
{
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
        let max = self
            .data
            .iter()
            .enumerate()
            .max_by_key(|probe| probe.1)
            .map(|probe| probe.0);
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
        }

        equal_with(sort_iterator) {
            fn drain(&mut self) -> impl Iterator<Item = T>;
        }
    }
}

fn fuzz_cycle(data: &[u8]) -> arbitrary::Result<()> {
    use arbitrary::{Arbitrary, Unstructured};

    let mut ring = Unstructured::new(&data);
    let capacity: u8 = Arbitrary::arbitrary(&mut ring)?;

    let mut model = ModelBinaryHeap::<u16>::default();
    let mut tested = BinaryHeap::<u16>::with_capacity(capacity as usize);

    let mut _op_trace = String::new();
    while let Ok(op) = <op::Op<u16> as Arbitrary>::arbitrary(&mut ring) {
        op.append_to_trace(&mut _op_trace);
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
