# arbitrary-model-tests

This is an attempt at creating a convenient procedural macro to be used for testing stateful models (in particular, various kinds of data structures) against a trivial (but usually very inefficient) implementation that is semantically 100% equivalent and *obviously* correct at the same time. The purpose of the macro is to generate the inner loop logic so that the definition of a model test for a given stateful structure becomes as succinct as possible.

This crate was inspired by the following work:

[https://github.com/blt/bughunt-rust](bughunt-rust)
[https://github.com/rust-fuzz/cargo-fuzz](cargo-fuzz)
[https://github.com/rust-fuzz/honggfuzz-rs](honggfuzz-rs)

## Example

This is the initial take at a DSL that describes the stateful model to be tested (`std::collections::HashMap` in this case).

```rust
arbitrary_stateful_operations! {
    model = ModelHashMap<K, V>,
    tested = HashMap<K, V>,
    
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
            fn is_empty(&self) -> bool;
            fn len(&self) -> usize;
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
}
```
