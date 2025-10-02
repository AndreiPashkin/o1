//! Deriving a hasher for a primitive-only struct should succeed on the compile-time hashing path.
use o1::hashing::hashers::msp::MSPHasher;
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

const fn const_hash() -> u32 {
    let hasher = SimpleKeyHasher::from_seed_const(1, 8);
    hasher.hash_const(&SAMPLE_KEY)
}

const _CONST_HASH: u32 = const_hash();

fn main() {}
