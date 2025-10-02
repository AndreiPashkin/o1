//! Deriving a hasher for a user-defined enum with field overrides should succeed on the runtime
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
    prefix: &'static str,
    suffix: &'static str,
}

#[derive(PartialEq, Eq, Hasher)]
#[o1(hasher = MSPHasher)]
enum CompositeEnum {
    Number {
        #[o1(hasher = NumberHolderHasher)]
        inner: NumberHolder,
    },
    Numbers {
        #[o1(hasher = NumbersHolderHasher)]
        inner: NumbersHolder,
    },
    Message {
        #[o1(hasher = MessageHasher)]
        inner: Message,
    },
}

const CASES: [CompositeEnum; 3] = [
    CompositeEnum::Number {
        inner: NumberHolder { value: -7 },
    },
    CompositeEnum::Numbers {
        inner: NumbersHolder {
            values: [5, 6, 7, 8],
        },
    },
    CompositeEnum::Message {
        inner: Message {
            prefix: "foo",
            suffix: "bar",
        },
    },
];

fn main() {
    let hasher = CompositeEnumHasher::from_seed(2, 64);
    for case in CASES.iter() {
        let hash_value = hasher.hash(case);
        assert!(hash_value < 64);
    }
}
