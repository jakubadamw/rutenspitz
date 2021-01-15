//#![allow(clippy::let_unit_value)]
use honggfuzz::fuzz;
use rutenspitz::arbitrary_stateful_operations;

pub struct Extender<'a, T>(&'a mut Vec<T>);

impl<'a, T: Clone> Extender<'a, T> {
    fn extend_from_slice(&mut self, slice: &[T]) {
        self.0.extend_from_slice(slice);
    }
}

arbitrary_stateful_operations! {
    model = Extender<'a, T>,
    tested = Extender<'a, T>,

    type_parameters = <'a, T: Clone + std::fmt::Debug>,

    methods {
        equal {
            fn extend_from_slice(&mut self, sli: &[T]);
        }
    }
}

#[allow(clippy::unnecessary_wraps)]
fn fuzz_cycle(data: &[u8]) -> arbitrary::Result<()> {
    use arbitrary::{Arbitrary, Unstructured};

    let mut ring = Unstructured::new(&data);

    let mut vec = Vec::<u32>::new();
    let mut tested = Extender(&mut vec);

    let mut op_trace = String::new();
    while let Ok(op) = <op::Op<u32> as Arbitrary>::arbitrary(&mut ring) {
        op.append_to_trace(&mut op_trace);
        op.execute(&mut tested);
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
