//! Deriving a hasher for a primitive-only struct should succeed on the runtime hashing path.
use o1::hashing::hashers::msp::MSPHasher;
use o1_core::Hasher as _;
use o1_derive::Hasher;

#[derive(PartialEq, Eq, Hasher)]
#[o1(hasher = MSPHasher)]
struct SimpleKey {
    number: i64,
    numbers: [u32; 4],
    text: &'static str,
}

const SAMPLE_KEY: SimpleKey = SimpleKey {
    number: -42,
    numbers: [1, 2, 3, 4],
    text: "example",
};

fn main() {
    let hasher = SimpleKeyHasher::from_seed(1, 8);
    let runtime_hash = hasher.hash(&SAMPLE_KEY);
    assert!(runtime_hash < 8);
}
