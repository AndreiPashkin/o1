//! Using a derived enum hasher should allow building an FKS map via the runtime constructor.
use o1::fks::FKSMap;
use o1::hashing::hashers::msp::MSPHasher;
use o1_core::HashMap as _;
use o1_derive::Hasher;

#[derive(Debug, PartialEq, Eq, Hasher)]
#[o1(hasher = MSPHasher)]
enum EndpointKey {
    Root,
    User(u32),
    Region { code: u16, label: &'static str },
}

type DerivedEndpointHasher = EndpointKeyHasher<EndpointKey>;

fn main() {
    let data: Box<[(EndpointKey, u8)]> = Box::new([
        (EndpointKey::Root, 1),
        (EndpointKey::User(42), 2),
        (
            EndpointKey::Region {
                code: 7,
                label: "west",
            },
            3,
        ),
    ]);

    let map = FKSMap::<EndpointKey, u8, DerivedEndpointHasher>::new(data, 99, 0.7)
        .expect("FKSMap::new should succeed");

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
        assert_eq!(map.get(key), Some(value));
    }

    assert!(map.get(&EndpointKey::User(7)).is_none());
}
