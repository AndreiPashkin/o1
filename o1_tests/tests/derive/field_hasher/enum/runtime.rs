//! Field-level hasher overrides in an enum should work on the runtime hashing path.
use o1::hashing::hashers::msp::MSPHasher;
use o1::hashing::hashers::xxh3::XXH3Hasher;
use o1_core::Hasher as _;
use o1_derive::Hasher;

#[derive(PartialEq, Eq, Hasher)]
#[o1(hasher = MSPHasher)]
enum MixedHasherEnum {
    Simple(u32),
    Detailed {
        id: u32,
        #[o1(hasher = XXH3Hasher)]
        checksum: u64,
    },
}

fn main() {
    let cases = [
        MixedHasherEnum::Simple(3),
        MixedHasherEnum::Detailed {
            id: 5,
            checksum: 99,
        },
    ];

    let hasher = MixedHasherEnumHasher::from_seed(17, 128);
    for value in cases.iter() {
        let hash = hasher.hash(value);
        assert!(hash < 128);
    }
}
