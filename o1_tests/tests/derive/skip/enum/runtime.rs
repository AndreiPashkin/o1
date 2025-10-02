//! `#[o1(skip)]` should exclude enum fields from hashing when using the runtime hashing path.
use o1::hashing::hashers::msp::MSPHasher;
use o1_core::Hasher as _;
use o1_derive::Hasher;

#[derive(PartialEq, Eq, Hasher)]
#[o1(hasher = MSPHasher)]
enum SkippedEnum {
    Alpha {
        value: u32,
        #[o1(skip)]
        note: &'static str,
    },
    Beta(u32, #[o1(skip)] &'static str),
}

fn main() {
    let hasher = SkippedEnumHasher::from_seed(13, 64);

    let alpha_a = SkippedEnum::Alpha {
        value: 5,
        note: "first",
    };
    let alpha_b = SkippedEnum::Alpha {
        value: 5,
        note: "second",
    };
    assert_eq!(hasher.hash(&alpha_a), hasher.hash(&alpha_b));

    let beta_a = SkippedEnum::Beta(9, "inner");
    let beta_b = SkippedEnum::Beta(9, "different");
    assert_eq!(hasher.hash(&beta_a), hasher.hash(&beta_b));
}
