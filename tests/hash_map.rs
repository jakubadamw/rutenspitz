use std::collections::HashMap;

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
    let capacity: usize = 7;
    let hash_seed: u64 = 4774451669087367725;
    let mut v: HashMap<u16, u16, BuildAHasher> = HashMap::with_capacity_and_hasher(
        capacity,
        BuildAHasher::new(hash_seed),
    );

    v.insert(1988, 29987);
    v.insert(2666, 27242);
    v.insert(6040, 2394);
    v.insert(25752, 61248);
    v.insert(27146, 27242);
    v.insert(27241, 27242);
    v.insert(27242, 27242);
    v.insert(27243, 27242);
    v.insert(27285, 27242);
    v.insert(27331, 27242);
    v.insert(28712, 1989);
    v.insert(29517, 57394);
    v.insert(32582, 1480);
    v.insert(34410, 27242);
    v.insert(35690, 26931);
    v.insert(38250, 27242);
    v.insert(39274, 15180);
    v.insert(44843, 27864);
    v.insert(48680, 48830);
    v.insert(56389, 27242);
    v.insert(57394, 52917);
    v.insert(61248, 34543);
    v.insert(61510, 51837);
    v.insert(63016, 47943);
    v.remove(&29517);

    let previous_capacity = v.capacity();
    v.clear();
    assert_eq!(v.capacity(), previous_capacity, "v.capacity() != previous_capacity");
}
