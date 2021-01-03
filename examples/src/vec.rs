//#![allow(clippy::let_unit_value)]
use honggfuzz::fuzz;
use rutenspitz::arbitrary_stateful_operations;

arbitrary_stateful_operations! {
    model = Vec<T>,
    tested = Vec<T>,

    type_parameters = <T: Clone + std::fmt::Debug>,

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

    let mut tested = Vec::<u32>::new();

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
