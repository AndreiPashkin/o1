//! `#[o1(skip)]` should exclude struct fields from hashing when using the runtime hashing path.
use o1::hashing::hashers::msp::MSPHasher;
use o1_core::Hasher as _;
use o1_derive::Hasher;

#[derive(PartialEq, Eq, Hasher)]
#[o1(hasher = MSPHasher)]
struct SkippedStruct {
    id: u32,
    #[o1(skip)]
    memo: &'static str,
}

fn main() {
    let base = SkippedStruct {
        id: 7,
        memo: "alpha",
    };
    let variant = SkippedStruct {
        id: 7,
        memo: "beta",
    };

    let hasher = SkippedStructHasher::from_seed(11, 32);

    let base_hash = hasher.hash(&base);
    let variant_hash = hasher.hash(&variant);
    assert_eq!(
        base_hash, variant_hash,
        "skipped field should not affect the hash"
    );
}
