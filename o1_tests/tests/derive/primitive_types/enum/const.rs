//! Deriving a hasher for a primitive-only enum should hash all variants on the compile-time path.
use o1::hashing::hashers::msp::MSPHasher;
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

const fn hashes() -> [u32; 3] {
    let hasher = PrimitiveEnumHasher::from_seed_const(1, 16);
    let mut output = [0u32; 3];
    let mut idx = 0;
    while idx < SAMPLE_CASES.len() {
        output[idx] = hasher.hash_const(&SAMPLE_CASES[idx]);
        idx += 1;
    }
    output
}

const _CONST_HASHES: [u32; 3] = hashes();

fn main() {}
