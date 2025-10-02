//! Deriving a hasher for a user-defined enum with field overrides should succeed on the
//! compile-time hashing path.
use o1::hashing::hashers::msp::MSPHasher;
use o1_derive::Hasher;

#[derive(PartialEq, Eq, Clone, Hasher)]
#[o1(hasher = MSPHasher, type_name = "NumberHolderHasher")]
struct NumberHolder {
    value: i64,
}

#[derive(PartialEq, Eq, Clone, Hasher)]
#[o1(hasher = MSPHasher, type_name = "NumbersHolderHasher")]
struct NumbersHolder {
    values: [u32; 4],
}

#[derive(PartialEq, Eq, Clone, Hasher)]
#[o1(hasher = MSPHasher, type_name = "MessageHasher")]
struct Message {
    prefix: &'static str,
    suffix: &'static str,
}

#[derive(PartialEq, Eq, Clone, Hasher)]
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

const fn hashes() -> [u32; 3] {
    let hasher = CompositeEnumHasher::from_seed_const(2, 64);
    let mut output = [0u32; 3];
    let mut index = 0;
    while index < CASES.len() {
        output[index] = hasher.hash_const(&CASES[index]);
        index += 1;
    }
    output
}

const _CONST_HASHES: [u32; 3] = hashes();

fn main() {}
