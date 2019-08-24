#![allow(clippy::let_unit_value)]

#[macro_use]
extern crate arbitrary_model_tests;
#[macro_use]
extern crate derive_arbitrary;
#[macro_use]
extern crate honggfuzz;

use std::fmt::Debug;

trait UrlFix {
    fn set_fragment_(&mut self, fragment: &Option<String>);
    fn set_host_(&mut self, host: &Option<String>);
    fn set_password_(&mut self, password: &Option<String>);
    fn set_query_(&mut self, query: &Option<String>);
}

impl UrlFix for url::Url {
    fn set_fragment_(&mut self, fragment: &Option<String>) {
        self.set_fragment(fragment.as_ref().map(String::as_str));
    }

    fn set_host_(&mut self, host: &Option<String>) {
        let _ = self.set_host(host.as_ref().map(String::as_str));
    }

    fn set_password_(&mut self, password: &Option<String>) {
        let _ = self.set_password(password.as_ref().map(String::as_str));
    }

    fn set_query_(&mut self, query: &Option<String>) {
        let _ = self.set_query(query.as_ref().map(String::as_str));
    }
}

#[allow(dead_code)]
fn map_to_vec<T, I: Iterator<Item = T>>(i: Option<I>) -> Option<Vec<T>> {
    i.map(Iterator::collect)
}

arbitrary_stateful_operations! {
    model = url::Url,
    tested = url::Url,

    type_parameters = <>,

    methods {
        equal {
            fn as_str(&self) -> &str;
            fn cannot_be_a_base(&self) -> bool;
            fn domain(&self) -> Option<&str>;
            fn fragment(&self) -> Option<&fragment>;
            fn has_authority(&self) -> bool;
            fn has_host(&self) -> bool;
            fn host(&self) -> Option<url::Host<&str>>;
            fn host_str(&self) -> Option<&str>;
            fn join(&self, input: &String) -> Result<url::Url, url::ParseError>;
            fn origin(&self) -> url::Origin;
            fn password(&self) -> Option<&str>;
            fn path(&self) -> &str;
            fn port(&self) -> Option<u16>;
            fn port_or_known_default(&self) -> Option<u16>;
            fn query(&self) -> Option<&str>;
            fn scheme(&self) -> &str;
            fn set_fragment_(&mut self, fragment: &Option<String>);
            fn set_host_(&mut self, host: &Option<String>);
            fn set_password_(&mut self, password: &Option<String>);
            fn set_path(&mut self, path: &String);
            fn set_port(&mut self, port: Option<u16>);
            fn set_query_(&mut self, query: &Option<String>);
            fn set_scheme(&mut self, scheme: &String);
            fn set_username(&mut self, username: &String);
            fn to_file_path(&self) -> Result<std::path::PathBuf, ()>;
            fn username(&self) -> &str;
        }

        equal_with(std::iter::Iterator::collect::<Vec<_>>) {
            fn query_pairs(&self) -> url::Parse;
        }

        equal_with(map_to_vec) {
            fn path_segments(&self) -> Option<std::str::Split<char>>;
        }
    }
}

const MAX_RING_SIZE: usize = 65_536;

fn fuzz_cycle(data: &[u8]) -> Result<(), ()> {
    use arbitrary::{Arbitrary, FiniteBuffer};

    let mut ring = FiniteBuffer::new(&data, MAX_RING_SIZE).map_err(|_| ())?;

    let mut tested = url::Url::parse("https://example.org").unwrap();

    let mut _op_trace = String::new();
    while let Ok(op) = <op::Op as Arbitrary>::arbitrary(&mut ring) {
        #[cfg(fuzzing_debug)]
        _op_trace.push_str(&format!("{}\n", op.to_string()));
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
