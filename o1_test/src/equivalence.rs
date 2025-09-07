//! Implements a utility-function for testing equivalence between hash function families.
use rand::prelude::*;
use std::fmt::Debug;

type HashFunctionFamily<K> = dyn Fn(u64, usize) -> (Box<dyn Fn(&K) -> usize>, usize);

/// Verifies that two hash function families produce identical outputs for the same inputs.
///
/// # Parameters
///
/// - `rng`: A random number generator used to create test keys and seeds.
/// - `family1`: The first hash function family to test.
///              Takes a seed and requested bucket count, returns a hash function
///              and actual bucket count.
/// - `family2`: The second hash function family to test (e.g., const fn implementation).
/// - `gen_key`: Key generator function.
/// - `raw_num_buckets`: The requested number of buckets to use for testing.
/// - `num_trials`: Number of different (seed, key) pairs to test.
///
/// # Panics
///
/// - If two families resolve to different bucket counts
/// - For any (seed, key) pair, the hash values differ between implementations
pub fn equivalence<R, K>(
    rng: &mut R,
    family1: &HashFunctionFamily<K>,
    family2: &HashFunctionFamily<K>,
    gen_key: &dyn Fn(&mut R) -> K,
    raw_num_buckets: usize,
    num_trials: usize,
) where
    R: Rng,
    K: PartialEq + Debug,
{
    let (_, num_buckets1) = family1(0, raw_num_buckets);
    let (_, num_buckets2) = family2(0, raw_num_buckets);

    assert_eq!(
        num_buckets1, num_buckets2,
        "Hash function families resolve different number of buckets: {}, {}",
        num_buckets1, num_buckets2
    );

    for _ in 0..num_trials {
        let seed = rng.next_u64();

        let (hash_fn1, _) = family1(seed, raw_num_buckets);
        let (hash_fn2, _) = family2(seed, raw_num_buckets);

        let key = gen_key(rng);

        let hash1 = hash_fn1(&key);
        let hash2 = hash_fn2(&key);

        assert_eq!(
            hash1, hash2,
            "Hash functions produce different results for seed {}, key {:?}: {}, {}",
            seed, key, hash1, hash2,
        );
    }
}

/// Generalizes hasher class equivalence testing.
#[macro_export]
macro_rules! hasher_equivalence {
    ($H:ty, $K:ty, $rng: expr, $gen_key:expr, $raw_num_buckets:expr, $num_trials:expr) => {{
        use rand::Rng;
        use std::fmt::Debug;
        use $crate::equivalence::equivalence;

        pub fn _hasher_equivalence<R>(
            rng: &mut R,
            gen_key: &dyn Fn(&mut R) -> $K,
            raw_num_buckets: usize,
            num_trials: usize,
        ) where
            R: Rng,
            $K: PartialEq + Debug,
        {
            let family1 = |seed: u64, num_buckets: usize| {
                let seed = seed | 1;
                let state = <$H>::make_state(seed, num_buckets as u32);
                let hasher = <$H>::from_state(state.clone());

                (
                    Box::new(move |value: &$K| {
                        let h = <$H>::from_state(state);
                        h.hash(value) as usize
                    }) as Box<dyn Fn(&$K) -> usize>,
                    hasher.num_buckets() as usize,
                )
            };
            let family2 = |seed: u64, num_buckets: usize| {
                let seed = seed | 1;
                let state = <$H>::make_state(seed, num_buckets as u32);
                let hasher = <$H>::from_state(state.clone());

                (
                    Box::new(move |value: &$K| {
                        let h = <$H>::from_state(state);
                        h.hash_const(value) as usize
                    }) as Box<dyn Fn(&$K) -> usize>,
                    hasher.num_buckets() as usize,
                )
            };

            equivalence::<R, $K>(
                rng,
                &family1,
                &family2,
                gen_key,
                raw_num_buckets,
                num_trials,
            );
        }

        _hasher_equivalence($rng, &$gen_key, $raw_num_buckets, $num_trials)
    }};
}
pub use hasher_equivalence;
