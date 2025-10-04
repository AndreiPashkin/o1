//! Field-level hasher overrides in an enum should work on the compile-time hashing path.
use o1::hashing::hashers::msp::MSPHasher;
use o1::hashing::hashers::xxh3::XXH3Hasher;
use o1_derive::Hasher;

#[derive(PartialEq, Eq, Clone, Copy, Hasher)]
#[o1(hasher = MSPHasher)]
enum MixedHasherEnum {
    Simple(u32),
    Detailed {
        id: u32,
        #[o1(hasher = XXH3Hasher)]
        checksum: u64,
    },
}

const CASES: [MixedHasherEnum; 2] = [
    MixedHasherEnum::Simple(10),
    MixedHasherEnum::Detailed {
        id: 12,
        checksum: 88,
    },
];

const fn hash_enum() -> [u32; 2] {
    let hasher = MixedHasherEnumHasher::from_seed_const(19, 256);
    let mut results = [0u32; 2];
    let mut i = 0;
    while i < CASES.len() {
        results[i] = hasher.hash_const(&CASES[i]);
        i += 1;
    }
    results
}

const _: [u32; 2] = hash_enum();

fn main() {}
