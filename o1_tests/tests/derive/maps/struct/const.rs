//! Using a derived struct hasher should allow building an FKS map via the compile-time
//! constructor.
use o1::hashing::hashers::msp::MSPHasher;
use o1::new_fks_map;
use o1_core::HashMap as _;
use o1_derive::Hasher;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hasher)]
#[o1(hasher = MSPHasher)]
struct RouteKey {
    code: u16,
    region: &'static str,
}

type DerivedRouteHasher = RouteKeyHasher<RouteKey>;

const ROUTE_DATA: [(RouteKey, u8); 3] = [
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
];

new_fks_map!(
    ROUTE_MAP,
    RouteKey,
    u8,
    ROUTE_DATA,
    DerivedRouteHasher,
    42,
    0.7
);

fn main() {
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

    for (key, value) in expected.iter() {
        assert_eq!(ROUTE_MAP.get(key), Some(value));
    }

    let missing = RouteKey {
        code: 404,
        region: "east",
    };
    assert!(ROUTE_MAP.get(&missing).is_none());
}
