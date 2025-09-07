/// Alternative compile-time constructor capable of creating static [`FKSMap`] instances.
///
/// # Parameters
///
/// - `name`: The name of the resulting static variable.
/// - `data`: The data to be hashed.
/// - `hasher_type`: Hasher type that should be used to hash the keys.
/// - `seed`: The seed for the random number generator.
/// - `min_load_factor`: The minimum load factor.
///
/// # Examples
///
/// ```rust
/// use o1_core::HashMap;
/// use o1::hashing::hashers::msp::MSPHasher;
/// use o1::new_fks_map;
///
/// // Create a static perfect hash map of book ratings
/// new_fks_map!(BOOK_RATINGS, &'static str, u8, [
///     ("The Great Gatsby", 5),
///     ("Moby Dick", 4),
///     ("Pride and Prejudice", 5),
///     ("The Catcher in the Rye", 3),
/// ], MSPHasher<&'static str>, 42, 0.75);
///
/// // Now you can use the static map
/// assert_eq!(BOOK_RATINGS.get(&"Moby Dick"), Some(&4));
/// assert_eq!(BOOK_RATINGS.get(&"The Great Gatsby"), Some(&5));
/// assert_eq!(BOOK_RATINGS.get(&"War and Peace"), None);
/// ```
#[doc(hidden)]
#[macro_export]
macro_rules! new_fks_map {
    ($name:ident, $K:ty, $V:ty, $data:expr, $HasherType:ty, $seed:expr, $min_load_factor:expr$(,)?) => {
        static $name: $crate::fks::FKSMap<'static, $K, $V, $HasherType> = {
            use core::marker::PhantomData;
            use core::mem::{swap, transmute_copy, MaybeUninit};
            use o1_core::Hasher;
            use $crate::fks::{Bucket, FKSMap};
            use $crate::utils::bit_array::{BitArray, Bits};
            use $crate::utils::const_hacks::div_ceil_f32;
            use $crate::utils::maybe_owned_slice::MaybeOwnedSliceMut;
            use $crate::utils::xorshift::XorShift;

            const MAX_KEYS_PER_BUCKET: usize = 5;
            const MAX_NUM_BUCKETS: usize =
                div_ceil_f32($data.len() as f32, $min_load_factor as f32) as usize;
            const DATA_LEN: usize = $data.len();
            const DATA_REF: &[($K, $V); DATA_LEN] = &($data);
            const KEY_BIT_ARRAY_LEN: usize = div_ceil_f32($data.len() as f32, 64 as f32) as usize;

            /// A compile-time alternative bucket type of the hash table.
            #[derive(Clone)]
            #[doc(hidden)]
            pub struct ConstBucket {
                /// The offset of the first slot in the bucket.
                pub offset: usize,
                /// A bit-mask of the occupied slots in the bucket.
                pub slots: u8,
                /// A number of slots in the bucket.
                pub num_slots: u8,
                /// L2 hasher that contains parameters for the L2 hash function.
                pub hasher: $HasherType,
                pub hasher_state: <$HasherType as Hasher<$K>>::State,
            }

            /// Result of resolving L1 and L2 hash functions.
            ///
            /// It's an intermediate result of constructing the hash table. It contains everything
            /// that is necessary to build the table, but the `l1_hasher` contains a const-version
            /// of the hasher, `buckets` contains const-buckets and of unoptimal size.
            #[doc(hidden)]
            struct ResolveResult<const MAX_NUM_BUCKETS: usize> {
                /// Total number of data-slots in the hash table.
                num_slots: usize,
                /// Total number of buckets in the hash table. Supposed to be less than or equal to
                /// `MAX_NUM_BUCKETS`.
                num_buckets: usize,
                l1_hasher: $HasherType,
                /// Buckets of the hash-table.
                buckets: [MaybeUninit<ConstBucket>; MAX_NUM_BUCKETS],
            }

            /// Contains all the data required to instantiate the static [`FKSMap`].
            struct BuildResult<const NUM_BUCKETS: usize, const NUM_SLOTS: usize> {
                /// Non-const L1-hasher.
                l1_hasher: $HasherType,
                /// Array of non-const buckets of optimal size.
                buckets: [Bucket<$K, $HasherType>; NUM_BUCKETS],
                /// Data-array of optimal size.
                slots: [MaybeUninit<($K, $V)>; NUM_SLOTS],
            }

            /// Attempts to find a suitable level-1 hash function for the given input data.
            ///
            /// # Parameters
            ///
            /// - `rng`: The random number generator.
            /// - `load_factor`: The desired load factor - it would determined the size of the output
            ///                  space of the hash function.
            /// - `num_trials`: The number of trials to attempt.
            /// - `data`: The input dataset in form of a slice of key-value pairs as tuples.
            /// - `DATA_LEN`: The length of the input dataset.
            /// - `MAX_NUM_BUCKETS`: The maximum possible number of buckets under the minimum load
            ///                      factor (not to confuse with the current load factor passed as
            ///                      `load_factor`).
            const fn try_resolve_l1<
                const DATA_LEN: usize,
                const MAX_NUM_BUCKETS: usize,
                const KEY_BIT_ARRAY_LEN: usize,
            >(
                rng: &mut XorShift<u64>,
                load_factor: f32,
                num_trials: usize,
                data: &[($K, $V); DATA_LEN],
            ) -> Option<(
                $HasherType,
                [BitArray<u64, KEY_BIT_ARRAY_LEN>; MAX_NUM_BUCKETS],
            )> {
                let mut trial_idx = 0;
                while trial_idx < num_trials {
                    let num_buckets_raw = div_ceil_f32(DATA_LEN as f32, load_factor) as u32;
                    let l1_hasher = <$HasherType>::from_seed_const(rng.next(), num_buckets_raw);
                    let num_buckets = l1_hasher.num_buckets_const() as usize;

                    if num_buckets > MAX_NUM_BUCKETS {
                        break;
                    }

                    let mut bucket_to_keys: [BitArray<u64, KEY_BIT_ARRAY_LEN>; MAX_NUM_BUCKETS] =
                        { [BitArray::<u64, KEY_BIT_ARRAY_LEN>::new(); MAX_NUM_BUCKETS] };

                    let mut i = 0;
                    while i < DATA_LEN {
                        let hash = l1_hasher.hash_const(&data[i].0) as usize;
                        bucket_to_keys[hash].set(i);
                        i += 1;
                    }

                    let mut max_keys_per_bucket: usize = 0;
                    let mut i = 0;
                    while i < num_buckets {
                        let num_keys = bucket_to_keys[i].count_ones();
                        if num_keys > max_keys_per_bucket {
                            max_keys_per_bucket = num_keys;
                        }
                        i += 1;
                    }

                    if max_keys_per_bucket <= MAX_KEYS_PER_BUCKET {
                        return Some((l1_hasher, bucket_to_keys));
                    }

                    trial_idx += 1;
                }

                None
            }

            /// Attempt to find the L2 hash function for the given bucket.
            ///
            /// # Parameters
            ///
            /// - `rng`: A random number generator.
            /// - `bucket_idx`: The global index of the bucket in the L1 table.
            /// - `current_offset`: The current global offset of the bucket.
            /// - `data`: The complete input data.
            /// - `bucket_to_keys`: An array of bit-arrays where each bit-array is a bucket
            ///                     and each bit is a key in the input data.
            /// - `num_trials`: The maximum number of trials to find the hash function.
            /// - `DATA_LEN`: The length of the input dataset.
            /// - `MAX_NUM_BUCKETS`: The maximum possible number of buckets under the minimum load
            ///                      factor.
            const fn try_resolve_bucket<
                const DATA_LEN: usize,
                const MAX_NUM_BUCKETS: usize,
                const KEY_BIT_ARRAY_LEN: usize,
            >(
                rng: &mut XorShift<u64>,
                bucket_idx: usize,
                current_offset: usize,
                data: &[($K, $V); DATA_LEN],
                bucket_to_keys: &[BitArray<u64, KEY_BIT_ARRAY_LEN>; MAX_NUM_BUCKETS],
                num_trials: usize,
            ) -> Option<ConstBucket> {
                let keys = &bucket_to_keys[bucket_idx];
                let num_keys: usize = keys.count_ones();

                if num_keys == 0 {
                    return Some(ConstBucket {
                        offset: 0,
                        slots: 0,
                        num_slots: 0,
                        hasher: <$HasherType>::from_seed_const(1, 1),
                        hasher_state: <$HasherType>::make_state_const(1, 1),
                    });
                }

                let mut trial_idx = 0;
                while trial_idx < num_trials {
                    let seed = rng.next();
                    let l2_hasher = <$HasherType>::from_seed_const(seed, num_keys as u32);
                    let num_slots = l2_hasher.num_buckets_const() as u8;

                    if num_slots > u8::MAX {
                        panic!("Number of slots exceeds u8::MAX");
                    }

                    let mut slots: Bits<u8> = Bits::<u8>::new();
                    let mut is_collision = false;
                    let mut iter = keys.iter_ones_const();

                    while let Some(key_idx) = iter.next() {
                        let hash = l2_hasher.hash_const(&data[key_idx].0) as usize;

                        if slots.get(hash).unwrap() {
                            is_collision = true;
                            break;
                        }
                        slots.set(hash);
                    }

                    if !is_collision {
                        return Some(ConstBucket {
                            offset: current_offset,
                            slots: slots.value(),
                            num_slots,
                            hasher: l2_hasher,
                            hasher_state: <$HasherType>::make_state_const(seed, num_keys as u32),
                        });
                    }

                    trial_idx += 1;
                }

                None
            }

            /// Attempts to resolve the level-1 hash function and the per-bucket level-2 hash functions.
            const fn try_resolve<const MAX_NUM_BUCKETS: usize, const KEY_BIT_ARRAY_LEN: usize>(
                data: &[($K, $V); DATA_LEN],
                seed: u64,
                min_load_factor: f32,
            ) -> Option<ResolveResult<MAX_NUM_BUCKETS>> {
                let mut rng = XorShift::<u64>::new(seed);

                let mut load_factor = 1.0;
                let mut l1_result = None;

                const MAX_L1_TRIALS: usize = 999;

                while load_factor >= min_load_factor {
                    l1_result = try_resolve_l1::<DATA_LEN, MAX_NUM_BUCKETS, KEY_BIT_ARRAY_LEN>(
                        &mut rng,
                        load_factor,
                        MAX_L1_TRIALS,
                        data,
                    );

                    if l1_result.is_some() {
                        break;
                    }

                    load_factor -= 0.05;
                }

                let (l1_hasher, bucket_to_keys) = match l1_result {
                    Some(result) => result,
                    None => return None,
                };

                let mut buckets: [MaybeUninit<ConstBucket>; MAX_NUM_BUCKETS] =
                    { unsafe { MaybeUninit::uninit().assume_init() } };
                let mut i = 0;
                while i < MAX_NUM_BUCKETS {
                    buckets[i] = MaybeUninit::new(ConstBucket {
                        offset: 0,
                        slots: 0,
                        num_slots: 0,
                        hasher: <$HasherType>::from_seed_const(1, 1),
                        hasher_state: <$HasherType>::make_state_const(1, 1),
                    });
                    i += 1;
                }

                let num_buckets = l1_hasher.num_buckets_const() as usize;
                let mut current_offset = 0;
                let mut bucket_idx = 0;

                const MAX_L2_TRIALS: usize = 999;

                while bucket_idx < num_buckets {
                    let bucket = try_resolve_bucket::<DATA_LEN, MAX_NUM_BUCKETS, KEY_BIT_ARRAY_LEN>(
                        &mut rng,
                        bucket_idx,
                        current_offset,
                        data,
                        &bucket_to_keys,
                        MAX_L2_TRIALS,
                    );
                    if bucket.is_none() {
                        return None;
                    }
                    let bucket = bucket.unwrap();

                    current_offset += bucket.num_slots as usize;
                    buckets[bucket_idx] = MaybeUninit::new(bucket);
                    bucket_idx += 1;
                }

                Some(ResolveResult {
                    num_slots: current_offset,
                    num_buckets,
                    l1_hasher,
                    buckets,
                })
            }

            // Builds [`BuildResult`] which contains everything to instantiate a static [`FKSMap`].
            const fn build<
                const NUM_BUCKETS: usize,
                const NUM_SLOTS: usize,
                const DATA_LEN: usize,
            >(
                data: [($K, $V); DATA_LEN],
                l1_hasher: $HasherType,
                const_buckets: [MaybeUninit<ConstBucket>; MAX_NUM_BUCKETS],
            ) -> BuildResult<NUM_BUCKETS, NUM_SLOTS> {
                let mut data: [MaybeUninit<($K, $V)>; DATA_LEN] = unsafe { transmute_copy(&data) };

                let mut buckets: [MaybeUninit<Bucket<$K, $HasherType>>; NUM_BUCKETS] =
                    { unsafe { MaybeUninit::uninit().assume_init() } };

                let mut slots: [MaybeUninit<($K, $V)>; NUM_SLOTS] =
                    { unsafe { MaybeUninit::uninit().assume_init() } };

                let mut i = 0;
                while i < NUM_BUCKETS {
                    let const_bucket = unsafe { const_buckets[i].assume_init_ref() };
                    buckets[i] = MaybeUninit::new(Bucket {
                        offset: const_bucket.offset,
                        slots: const_bucket.slots,
                        num_slots: const_bucket.num_slots,
                        hasher: <$HasherType>::from_state_const(const_bucket.hasher_state),
                        key_type: PhantomData,
                    });
                    i += 1;
                }

                let mut i = 0;
                while i < DATA_LEN {
                    let mut item: MaybeUninit<($K, $V)> = MaybeUninit::uninit();

                    swap(&mut item, &mut data[i]);

                    let (k, v) = unsafe { item.assume_init() };
                    // TODO: try to refactor to avoid redundant double-hasing.
                    let bucket_idx = l1_hasher.hash_const(&k) as usize;
                    let bucket = unsafe { const_buckets[bucket_idx].assume_init_ref() };
                    let slot_idx = bucket.hasher.hash_const(&k) as usize;
                    let data_idx = bucket.offset + slot_idx;

                    slots[data_idx] = MaybeUninit::new((k, v));

                    i += 1;
                }

                BuildResult {
                    l1_hasher,
                    buckets: unsafe { transmute_copy(&buckets) },
                    slots,
                }
            }

            // It is necessary to save the result as a global const-constant because it should be const
            // to use it's fields as sizes of the final arrays.
            //
            // It's impossible to do that within a scope of a function - hence the intermediate step.
            const RESOLVE_RESULT: ResolveResult<MAX_NUM_BUCKETS> =
                try_resolve::<MAX_NUM_BUCKETS, KEY_BIT_ARRAY_LEN>(
                    DATA_REF,
                    $seed,
                    $min_load_factor,
                )
                .expect("Unable to resolve the hash functions");

            // The results of the final step before intializing the map.
            const BUILD_RESULT: BuildResult<
                { RESOLVE_RESULT.num_buckets },
                { RESOLVE_RESULT.num_slots },
            > = {
                build::<{ RESOLVE_RESULT.num_buckets }, { RESOLVE_RESULT.num_slots }, DATA_LEN>(
                    *DATA_REF,
                    RESOLVE_RESULT.l1_hasher,
                    RESOLVE_RESULT.buckets,
                )
            };

            static mut BUCKETS: [Bucket<$K, $HasherType>; BUILD_RESULT.buckets.len()] =
                { BUILD_RESULT.buckets };
            static mut SLOTS: [MaybeUninit<($K, $V)>; BUILD_RESULT.slots.len()] =
                { BUILD_RESULT.slots };

            #[allow(static_mut_refs)]
            FKSMap::<'static, $K, $V, $HasherType> {
                l1_hasher: BUILD_RESULT.l1_hasher,
                buckets: MaybeOwnedSliceMut::Borrowed(unsafe { &mut BUCKETS }),
                slots: MaybeOwnedSliceMut::Borrowed(unsafe { &mut SLOTS }),
            }
        };
    };
}

#[allow(unused_imports)]
pub use new_fks_map as new_const;

#[cfg(test)]
mod tests {
    #![allow(long_running_const_eval)]
    use crate::hashing::hashers::msp::MSPHasher;
    use crate::new_fks_map;
    use o1_core::HashMap;
    use o1_testing::data::*;
    use o1_testing::generate_static_map_tests;

    new_fks_map!(U8_MAP, u8, u64, U8_DATA, MSPHasher<u8>, 42, 0.75);
    new_fks_map!(I8_MAP, i8, u64, I8_DATA, MSPHasher<i8>, 42, 0.75);
    new_fks_map!(U16_MAP, u16, u64, U16_DATA, MSPHasher<u16>, 42, 0.75);
    new_fks_map!(I16_MAP, i16, u64, I16_DATA, MSPHasher<i16>, 42, 0.75);
    new_fks_map!(U32_MAP, u32, u64, U32_DATA, MSPHasher<u32>, 42, 0.75);
    new_fks_map!(I32_MAP, i32, u64, I32_DATA, MSPHasher<i32>, 42, 0.75);
    new_fks_map!(U64_MAP, u64, u64, U64_DATA, MSPHasher<u64>, 42, 0.75);
    new_fks_map!(I64_MAP, i64, u64, I64_DATA, MSPHasher<i64>, 42, 0.75);
    new_fks_map!(U128_MAP, u128, u64, U128_DATA, MSPHasher<u128>, 42, 0.75);
    new_fks_map!(I128_MAP, i128, u64, I128_DATA, MSPHasher<i128>, 42, 0.75);
    new_fks_map!(
        STR_MAP,
        &'static str,
        u64,
        STR_DATA,
        MSPHasher<&'static str>,
        42,
        0.75,
    );

    generate_static_map_tests!(
        U8_MAP, U8_DATA, I8_MAP, I8_DATA, U16_MAP, U16_DATA, I16_MAP, I16_DATA, U32_MAP, U32_DATA,
        I32_MAP, I32_DATA, U64_MAP, U64_DATA, I64_MAP, I64_DATA, U128_MAP, U128_DATA, I128_MAP,
        I128_DATA, STR_MAP, STR_DATA,
    );
}
