# rutenspitz

**NOTE**: *This crate was previously called `arbitrary-model-tests`.*

[![Build status](https://github.com/jakubadamw/rutenspitz/workflows/Build/badge.svg)](https://github.com/jakubadamw/rutenspitz/actions?query=workflow%3ABuild)
[![crates.io](https://img.shields.io/crates/v/rutenspitz.svg)](https://crates.io/crates/rutenspitz)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

This is an attempt at creating a convenient procedural macro to be used for testing stateful models (in particular, various kinds of data structures) against a trivial (but usually very inefficient) implementation that is semantically 100% equivalent to the target implementation but, in contrast, *obviously* correct. The purpose of the macro is to generate the boilerplate code for testing particular operations of the model so that the user-provided definition of the test for a given stateful structure becomes as succinct as possible.

This crate was inspired by the following works:

* [bughunt-rust](https://github.com/blt/bughunt-rust)
* [cargo-fuzz](https://github.com/rust-fuzz/cargo-fuzz)
* [honggfuzz-rs](https://github.com/rust-fuzz/honggfuzz-rs)

## Example

See the [`HashMap` test](../examples/src/hash_map.rs) for reference.

You can run it with `cargo hfuzz`. First of all you'll need to install `honggfuzz` along with its system dependencies. See [this section](https://github.com/rust-fuzz/honggfuzz-rs#dependencies) for more details. When you're done, all you need to run the test:

```
cargo hfuzz run hash_map
```

## Debugging

The `Op` struct implements the `Display` trait by trying to produce actual Rust code equivalent of the operation (e.g. `v.insert(0, 2);` for an insert operation of a HashMap). This can be very usefully employed to produce a human-friendly operation trace to be inspected when debugging (in a proper debugger, say, lldb) an actual crash found by the fuzzer. See [these two lines](examples/src/hash_map.rs#L197-L198) for reference.

## DSL

This is the initial take at a DSL that describes the stateful model to be tested (`std::collections::HashMap` in this case).

```rust
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
```
