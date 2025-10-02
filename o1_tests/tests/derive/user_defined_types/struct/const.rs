//! Deriving a hasher for nested structs with field overrides should succeed on the compile-time
//! hashing path.
use o1::hashing::hashers::msp::MSPHasher;
use o1_derive::Hasher;

#[derive(PartialEq, Eq, Clone, Hasher)]
#[o1(hasher = MSPHasher)]
struct NumberHolder {
    value: i64,
}

#[derive(PartialEq, Eq, Clone, Hasher)]
#[o1(hasher = MSPHasher)]
struct NumbersHolder {
    values: [u32; 4],
}

#[derive(PartialEq, Eq, Clone, Hasher)]
#[o1(hasher = MSPHasher)]
struct Message {
    text: &'static str,
}

#[derive(PartialEq, Eq, Clone, Hasher)]
#[o1(hasher = MSPHasher)]
struct CompositeKey {
    #[o1(hasher = NumberHolderHasher)]
    number: NumberHolder,
    #[o1(hasher = NumbersHolderHasher)]
    numbers: NumbersHolder,
    #[o1(hasher = MessageHasher)]
    message: Message,
}

const SAMPLE_KEY: CompositeKey = CompositeKey {
    number: NumberHolder { value: -42 },
    numbers: NumbersHolder {
        values: [1, 2, 3, 4],
    },
    message: Message { text: "example" },
};

const fn const_hash() -> u32 {
    let hasher = CompositeKeyHasher::from_seed_const(1, 32);
    hasher.hash_const(&SAMPLE_KEY)
}

const _CONST_HASH: u32 = const_hash();

fn main() {}
