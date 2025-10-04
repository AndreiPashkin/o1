//! Using a derived enum hasher should allow building an FKS map via the compile-time constructor.
use o1::hashing::hashers::msp::MSPHasher;
use o1::new_fks_map;
use o1_core::HashMap as _;
use o1_derive::Hasher;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hasher)]
#[o1(hasher = MSPHasher)]
enum EndpointKey {
    Root,
    User(u32),
    Region { code: u16, label: &'static str },
}

type DerivedEndpointHasher = EndpointKeyHasher<EndpointKey>;

const ENDPOINT_DATA: [(EndpointKey, u8); 3] = [
    (EndpointKey::Root, 1),
    (EndpointKey::User(42), 2),
    (
        EndpointKey::Region {
            code: 7,
            label: "west",
        },
        3,
    ),
];

new_fks_map!(
    ENDPOINT_MAP,
    EndpointKey,
    u8,
    ENDPOINT_DATA,
    DerivedEndpointHasher,
    99,
    0.7
);

fn main() {
    let expected = [
        (EndpointKey::Root, 1u8),
        (EndpointKey::User(42), 2u8),
        (
            EndpointKey::Region {
                code: 7,
                label: "west",
            },
            3u8,
        ),
    ];

    for (key, value) in expected.iter() {
        assert_eq!(ENDPOINT_MAP.get(key), Some(value));
    }

    assert!(ENDPOINT_MAP.get(&EndpointKey::User(7)).is_none());
}
