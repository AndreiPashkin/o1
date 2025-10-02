//! `#[o1(skip)]` should exclude enum fields from hashing when using the compile-time hashing path.
use o1::hashing::hashers::msp::MSPHasher;
use o1_derive::Hasher;

#[derive(PartialEq, Eq, Clone, Hasher)]
#[o1(hasher = MSPHasher)]
enum SkippedEnum {
    Alpha {
        value: u32,
        #[o1(skip)]
        note: &'static str,
    },
    Beta(u32, #[o1(skip)] &'static str),
}

const fn hashes_ignore_skipped_fields() -> bool {
    let alpha_a = SkippedEnum::Alpha {
        value: 17,
        note: "note",
    };
    let alpha_b = SkippedEnum::Alpha {
        value: 17,
        note: "changed",
    };
    let beta_a = SkippedEnum::Beta(33, "memo");
    let beta_b = SkippedEnum::Beta(33, "other");

    let hasher = SkippedEnumHasher::from_seed_const(23, 64);

    let alpha_equal = hasher.hash_const(&alpha_a) == hasher.hash_const(&alpha_b);
    let beta_equal = hasher.hash_const(&beta_a) == hasher.hash_const(&beta_b);

    alpha_equal && beta_equal
}

const _: () = {
    assert!(hashes_ignore_skipped_fields());
};

fn main() {}
