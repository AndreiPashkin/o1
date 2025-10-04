//! `#[o1(skip)]` should exclude struct fields from hashing when using the compile-time hashing
//! path.
use o1::hashing::hashers::msp::MSPHasher;
use o1_derive::Hasher;

#[derive(PartialEq, Eq, Clone, Hasher)]
#[o1(hasher = MSPHasher)]
struct SkippedStruct {
    id: u32,
    #[o1(skip)]
    memo: &'static str,
}

const fn hashes_equal() -> bool {
    let a = SkippedStruct {
        id: 7,
        memo: "alpha",
    };
    let b = SkippedStruct {
        id: 7,
        memo: "beta",
    };
    let hasher = SkippedStructHasher::from_seed_const(11, 32);

    let hash_a = hasher.hash_const(&a);
    let hash_b = hasher.hash_const(&b);

    hash_a == hash_b
}

const _: () = {
    assert!(hashes_equal());
};

fn main() {}
