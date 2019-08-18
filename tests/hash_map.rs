use hashbrown::HashMap;

pub struct BuildAHasher {
    seed: u64,
}

impl BuildAHasher {
    pub fn new(seed: u64) -> Self {
        Self { seed }
    }
}

impl std::hash::BuildHasher for BuildAHasher {
    type Hasher = ahash::AHasher;

    fn build_hasher(&self) -> Self::Hasher {
        ahash::AHasher::new_with_key(self.seed)
    }
}

#[test]
fn test_capacity() {
    let capacity: usize = 28;
    let hash_seed: u64 = 4774451669087367725;
    let mut map = HashMap::with_capacity_and_hasher(capacity, BuildAHasher::new(hash_seed));

    let items: Vec<(u16, u16)> = vec![
        (1988, 29987),
        (2666, 27242),
        (6040, 2394),
        (25752, 61248),
        (27146, 27242),
        (27241, 27242),
        (27242, 27242),
        (27243, 27242),
        (27285, 27242),
        (27331, 27242),
        (28712, 1989),
        (29517, 57394),
        (32582, 1480),
        (34410, 27242),
        (35690, 26931),
        (38250, 27242),
        (39274, 15180),
        (44843, 27864),
        (48680, 48830),
        (56389, 27242),
        (57394, 52917),
        (61248, 34543),
        (61510, 51837),
        (63016, 47943),
    ];
    for (k, v) in items {
        map.insert(k, v);
        eprintln!("inserted {}\tcapacity = {}", k, map.capacity());
    }

    map.remove(&29517);
    eprintln!("removed 29517\tcapacity = {}", map.capacity());

    let previous_capacity = map.capacity();
    map.clear();
    assert_eq!(
        map.capacity(),
        previous_capacity,
        "map.capacity() != previous_capacity"
    );
}
