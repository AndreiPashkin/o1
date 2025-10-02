//! Deriving a hasher for nested structs with field overrides should succeed on the runtime
//! hashing path.
use o1::hashing::hashers::msp::MSPHasher;
use o1_core::Hasher as _;
use o1_derive::Hasher;

#[derive(PartialEq, Eq, Hasher)]
#[o1(hasher = MSPHasher, type_name = "NumberHolderHasher")]
struct NumberHolder {
    value: i64,
}

#[derive(PartialEq, Eq, Hasher)]
#[o1(hasher = MSPHasher, type_name = "NumbersHolderHasher")]
struct NumbersHolder {
    values: [u32; 4],
}

#[derive(PartialEq, Eq, Hasher)]
#[o1(hasher = MSPHasher, type_name = "MessageHasher")]
struct Message {
    text: &'static str,
}

#[derive(PartialEq, Eq, Hasher)]
#[o1(hasher = MSPHasher)]
struct CompositeKey {
    #[o1(hasher = NumberHolderHasher)]
    number: NumberHolder,
    #[o1(hasher = NumbersHolderHasher)]
    numbers: NumbersHolder,
    #[o1(hasher = MessageHasher)]
    message: Message,
}

fn main() {
    let key = CompositeKey {
        number: NumberHolder { value: -42 },
        numbers: NumbersHolder {
            values: [1, 2, 3, 4],
        },
        message: Message { text: "example" },
    };

    let hasher = CompositeKeyHasher::from_seed(1, 32);
    let hash_value = hasher.hash(&key);
    assert!(hash_value < 32);
}
