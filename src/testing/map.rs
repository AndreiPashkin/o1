//! Utilities for testing map implementations.
use crate::core::HashMap;
use crate::core::Hasher;
use crate::testing::Generate;
use rand::Rng;
use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::Hash;

/// Generates data suitable for passing to a map constructor.
pub fn generate_map_data<R: Rng, K: Eq + Hash + Generate<R>, V: Eq + Hash + Generate<R>>(
    rng: &mut R,
    size: usize,
    key_params: &K::GenerateParams,
    val_params: &V::GenerateParams,
) -> Box<[(K, V)]> {
    let keys = K::generate_many(rng, key_params, size).into_vec();
    let vals = V::generate_many(rng, val_params, size).into_vec();
    keys.into_iter()
        .zip(vals)
        .collect::<Vec<_>>()
        .into_boxed_slice()
}

/// Tests key retrieval for a given map.
pub fn test_get<
    R: Rng,
    K: Eq + Hash + Generate<R> + Debug,
    V: Copy + PartialEq + Debug,
    H: Hasher<K> + Debug,
    M: HashMap<K, V, H> + Debug,
>(
    rng: &mut R,
    map: M,
    data: &[(K, V)],
) {
    let keys: HashSet<&K> = data.iter().map(|(k, _)| k).collect();

    for (key, val) in data {
        assert_eq!(map.get(key), Some(val), "Key: {:?}", key);
    }
    let mut non_exitent_keys = Vec::new();
    for _ in 0..data.len().div_ceil(3) {
        loop {
            let key = K::generate(rng, &<K as Generate<R>>::GenerateParams::default());
            if !keys.contains(&key) {
                non_exitent_keys.push(key);
                break;
            }
        }
    }
    for key in non_exitent_keys {
        assert_eq!(map.get(&key), None, "Key: {:?}", key);
    }
}

/// Tests construction of a map.
pub fn test_build<
    K: Eq + Debug,
    V: Copy + Debug,
    H: Hasher<K>,
    M: HashMap<K, V, H>,
    C: Fn(Box<[(K, V)]>) -> M,
>(
    cons: C,
    data: Box<[(K, V)]>,
) -> M {
    cons(data)
}

/// Generates tests for a map type for integer keys.
#[macro_export]
macro_rules! generate_map_int_tests {
    ($Map:tt, $Hasher:tt, $cons: expr, $type:ty) => {
        compose_idents!(test_fn = [test_build_get_map_, $type], {
            #[test]
            fn test_fn() {
                use std::ops::Div;
                use $crate::testing::*;

                use rand::rngs::ThreadRng;
                let mut rng = rand::rng();

                let map_size: usize = if <$type>::BITS >= u32::BITS {
                    999
                } else {
                    (1_usize << <$type>::BITS).div(2).min(999)
                };

                let data = generate_map_data::<_, $type, u128>(
                    &mut rng,
                    map_size,
                    &<$type as Generate<ThreadRng>>::GenerateParams::default(),
                    &<u128 as Generate<ThreadRng>>::GenerateParams::default(),
                );
                let map = test_build::<
                    $type,
                    u128,
                    $Hasher<$type>,
                    $Map<$type, u128, $Hasher<$type>>,
                    _,
                >($cons, data.to_vec().into_boxed_slice());
                test_get(&mut rng, map, &data);
            }
        });
    };
}

/// Generates tests of special cases for a map type for integer keys.
#[macro_export]
macro_rules! generate_map_int_special_tests {
    ($Map:tt, $Hasher:tt, $cons: expr, $($type:ty),*) => {
        $(
            compose_idents!(test_fn = [test_get_key_zero_, $type], {
                #[test]
                fn test_fn() {
                    use std::ops::Div;
                    use $crate::testing::*;

                    use rand::rngs::ThreadRng;
                    let mut rng = rand::rng();

                    let map_size: usize = if <$type>::BITS >= u32::BITS {
                        999
                    } else {
                        (1_usize << <$type>::BITS).div(2).min(999)
                    };

                    for _ in 0..99 {
                        let data: Vec<($type, u128)> = generate_map_data::<_, $type, u128>(
                            &mut rng,
                            map_size,
                            &<$type as Generate<ThreadRng>>::GenerateParams::default(),
                            &<u128 as Generate<ThreadRng>>::GenerateParams::default(),
                        ).iter().filter(|&item| item.0 != 0 as $type).copied().collect::<Vec<($type, u128)>>();
                        let map = test_build::<$type, u128, $Hasher<$type>, $Map<$type, u128, $Hasher<$type>>, _>(
                            $cons,
                            data.into_boxed_slice(),
                        );

                        assert_eq!(map.get(&(0 as $type)), None);
                    }
                }
            });
        )*
    };
}

/// Generates tests for a map type for string keys.
#[macro_export]
macro_rules! generate_map_str_tests {
    ($Map:tt, $Hasher:tt, $factory: expr) => {
        #[test]
        fn test_build_get_map_str() {
            use $crate::testing::*;

            use rand::rngs::ThreadRng;

            let mut rng = rand::rng();
            let data = generate_map_data::<_, String, u128>(
                &mut rng,
                999,
                &<String as Generate<ThreadRng>>::GenerateParams::default(),
                &<u128 as Generate<ThreadRng>>::GenerateParams::default(),
            );
            let map = test_build::<
                String,
                u128,
                $Hasher<String>,
                $Map<String, u128, $Hasher<String>>,
                _,
            >($factory, data.to_vec().into_boxed_slice());
            test_get(&mut rng, map, &data);
        }
    };
}

/// Generates tests of special cases for a map type for string keys.
#[macro_export]
macro_rules! generate_map_str_special_tests {
    ($Map:tt, $Hasher:tt, $factory: expr) => {
        #[test]
        fn test_get_key_zero_str() {
            use $crate::testing::*;

            use rand::rngs::ThreadRng;

            let mut rng = rand::rng();

            for _ in 0..99 {
                let data: Vec<(String, u128)> = generate_map_data::<_, String, u128>(
                    &mut rng,
                    999,
                    &<String as Generate<ThreadRng>>::GenerateParams::default(),
                    &<u128 as Generate<ThreadRng>>::GenerateParams::default(),
                )
                .iter()
                .filter(|&s| s.0.len() != 0)
                .cloned()
                .collect();
                let map = test_build::<
                    String,
                    u128,
                    $Hasher<String>,
                    $Map<String, u128, $Hasher<String>>,
                    _,
                >($factory, data.into_boxed_slice());
                assert_eq!(map.get(&"".to_string()), None);
            }
        }
    };
}

/// Generates tests for a map type for string keys.
#[macro_export]
macro_rules! generate_map_tests {
    ($Map:tt, $Hasher:tt, $factory:expr) => {
        use compose_idents::compose_idents;
        use $crate::generate_map_int_special_tests;
        use $crate::generate_map_int_tests;
        use $crate::generate_map_str_special_tests;
        use $crate::generate_map_str_tests;

        generate_map_int_tests!($Map, $Hasher, $factory, u8);
        generate_map_int_tests!($Map, $Hasher, $factory, i8);
        generate_map_int_tests!($Map, $Hasher, $factory, u16);
        generate_map_int_tests!($Map, $Hasher, $factory, i16);
        generate_map_int_tests!($Map, $Hasher, $factory, u32);
        generate_map_int_tests!($Map, $Hasher, $factory, i32);
        generate_map_int_tests!($Map, $Hasher, $factory, u64);
        generate_map_int_tests!($Map, $Hasher, $factory, i64);
        generate_map_int_tests!($Map, $Hasher, $factory, u128);
        generate_map_int_tests!($Map, $Hasher, $factory, i128);
        generate_map_int_special_tests!(
            $Map, $Hasher, $factory, u8, i8, u16, i16, u32, i32, u64, i64, u128, i128
        );
        generate_map_str_tests!($Map, $Hasher, $factory);
        generate_map_str_special_tests!($Map, $Hasher, $factory);
    };
}

/// Generates tests for a static map.
///
/// It is supposed to be used in combination with pre-defined datasets from [`o1::testing::data`]
/// used to instantiate the static maps before invoking this macro.
#[macro_export]
macro_rules! generate_static_map_tests {
    (
        $u8_map:expr,
        $u8_data:expr,
        $i8_map:expr,
        $i8_data:expr,
        $u16_map:expr,
        $u16_data:expr,
        $i16_map:expr,
        $i16_data:expr,
        $u32_map:expr,
        $u32_data:expr,
        $i32_map:expr,
        $i32_data:expr,
        $u64_map:expr,
        $u64_data:expr,
        $i64_map:expr,
        $i64_data:expr,
        $u128_map:expr,
        $u128_data:expr,
        $i128_map:expr,
        $i128_data:expr,
        $str_map:expr,
        $str_data:expr$(,)?
    ) => {
        use compose_idents::compose_idents;

        macro_rules! generate_tests_for_type {
            ($type:ident, $map:expr, $data:expr) => {
                compose_idents!(test_fn = [test_static_map_, $type], {
                    #[test]
                    fn test_fn() {
                        for (key, val) in &$data {
                            assert_eq!($map.get(key), Some(val), "Key: {:?}", key);
                        }
                    }
                });
            };
        }

        generate_tests_for_type!(u8, $u8_map, $u8_data);
        generate_tests_for_type!(i8, $i8_map, $i8_data);
        generate_tests_for_type!(u16, $u16_map, $u16_data);
        generate_tests_for_type!(i16, $i16_map, $i16_data);
        generate_tests_for_type!(u32, $u32_map, $u32_data);
        generate_tests_for_type!(i32, $i32_map, $i32_data);
        generate_tests_for_type!(u64, $u64_map, $u64_data);
        generate_tests_for_type!(i64, $i64_map, $i64_data);
        generate_tests_for_type!(u128, $u128_map, $u128_data);
        generate_tests_for_type!(i128, $i128_map, $i128_data);
        generate_tests_for_type!(str, $str_map, $str_data);
    };
}
