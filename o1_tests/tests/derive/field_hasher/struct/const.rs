//! Field-level hasher overrides in a struct should work on the compile-time hashing path.
use o1::hashing::hashers::msp::MSPHasher;
use o1::hashing::hashers::xxh3::XXH3Hasher;
use o1_derive::Hasher;

#[derive(PartialEq, Eq, Clone, Copy, Hasher)]
#[o1(hasher = MSPHasher)]
struct MixedHasherStruct {
    primary: u64,
    #[o1(hasher = XXH3Hasher)]
    secondary: u64,
    label: &'static str,
}

const SAMPLE: MixedHasherStruct = MixedHasherStruct {
    primary: 1,
    secondary: 2,
    label: "const",
};

const fn hash_struct() -> u32 {
    let hasher = MixedHasherStructHasher::from_seed_const(7, 64);
    hasher.hash_const(&SAMPLE)
}

const _: u32 = hash_struct();

fn main() {}
