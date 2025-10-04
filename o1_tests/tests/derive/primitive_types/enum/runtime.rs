//! Deriving a hasher for a primitive-only enum should hash all variants on the runtime path.
use o1::hashing::hashers::msp::MSPHasher;
use o1_core::Hasher as _;
use o1_derive::Hasher;

#[derive(PartialEq, Eq, Hasher)]
#[o1(hasher = MSPHasher)]
enum PrimitiveEnum {
    Number(i64),
    Numbers([u32; 4]),
    Message {
        prefix: &'static str,
        suffix: &'static str,
    },
}

const SAMPLE_CASES: [PrimitiveEnum; 3] = [
    PrimitiveEnum::Number(-99),
    PrimitiveEnum::Numbers([10, 20, 30, 40]),
    PrimitiveEnum::Message {
        prefix: "hello",
        suffix: "world",
    },
];

fn main() {
    let hasher = PrimitiveEnumHasher::from_seed(1, 16);
    for case in SAMPLE_CASES.iter() {
        let hash_value = hasher.hash(case);
        assert!(hash_value < 16);
    }
}
