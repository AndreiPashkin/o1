//! Using a derived struct hasher should allow building an FKS map via the runtime constructor.
use o1::fks::FKSMap;
use o1::hashing::hashers::msp::MSPHasher;
use o1_core::HashMap as _;
use o1_derive::Hasher;

#[derive(Debug, PartialEq, Eq, Hasher)]
#[o1(hasher = MSPHasher)]
struct RouteKey {
    code: u16,
    region: &'static str,
}

fn main() {
    let data: Box<[(RouteKey, u8)]> = Box::new([
        (
            RouteKey {
                code: 101,
                region: "north",
            },
            3,
        ),
        (
            RouteKey {
                code: 202,
                region: "west",
            },
            7,
        ),
        (
            RouteKey {
                code: 303,
                region: "south",
            },
            11,
        ),
    ]);

    type DerivedRouteHasher = RouteKeyHasher<RouteKey>;

    let map = FKSMap::<RouteKey, u8, DerivedRouteHasher>::new(data, 42, 0.7)
        .expect("FKSMap::new should succeed for a tiny dataset");

    let expected = [
        (
            RouteKey {
                code: 101,
                region: "north",
            },
            3u8,
        ),
        (
            RouteKey {
                code: 202,
                region: "west",
            },
            7u8,
        ),
        (
            RouteKey {
                code: 303,
                region: "south",
            },
            11u8,
        ),
    ];

    for (key, value) in expected {
        assert_eq!(map.get(&key), Some(&value));
    }

    let missing = RouteKey {
        code: 404,
        region: "east",
    };
    assert!(map.get(&missing).is_none());
}
