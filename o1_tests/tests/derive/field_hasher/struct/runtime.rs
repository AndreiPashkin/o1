//! Field-level hasher overrides in a struct should work on the runtime hashing path.
use o1::hashing::hashers::msp::MSPHasher;
use o1::hashing::hashers::xxh3::XXH3Hasher;
use o1_core::Hasher as _;
use o1_derive::Hasher;

#[derive(PartialEq, Eq, Hasher)]
#[o1(hasher = MSPHasher)]
struct MixedHasherStruct {
    primary: u64,
    #[o1(hasher = XXH3Hasher)]
    secondary: u64,
    label: &'static str,
}

fn main() {
    let key = MixedHasherStruct {
        primary: 11,
        secondary: 22,
        label: "mixed",
    };

    let hasher = MixedHasherStructHasher::from_seed(5, 32);
    let hash_value = hasher.hash(&key);
    assert!(hash_value < 32);
}
