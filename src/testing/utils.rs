use rand::prelude::*;
use std::collections::HashMap;
use std::hash::Hash;

/// Generate `unique_keys` distributed over `num_buckets` using `hash_fn`.
pub fn get_hash_value_distribution<K: Clone>(
    unique_keys: &Vec<K>,
    num_buckets: usize,
    hash_fn: impl Fn(&K) -> usize,
) -> Vec<Vec<K>> {
    let mut buckets: Vec<Vec<K>> = vec![Vec::<K>::new(); num_buckets];

    for key in unique_keys {
        let bucket = hash_fn(key);
        buckets[bucket].push(key.clone());
    }

    buckets
}

/// Calculate a number of collisions in a hash value distribution.
pub fn get_num_collisions<K>(buckets: &Vec<Vec<K>>) -> usize {
    buckets
        .iter()
        .map(|keys| if keys.len() > 1 { 1 } else { 0 })
        .sum::<usize>()
}

/// Calculate a probability of collisions for hash-function family for given `keys`.
pub fn get_collision_prob<K: Clone>(
    keys: &Vec<K>,
    family: &dyn Fn(u64, usize) -> Box<dyn Fn(&K) -> usize>,
    num_trials: usize,
) -> (f64, usize) {
    let num_buckets = keys.len().next_power_of_two();
    let mut total_num_collisions: u64 = 0;

    for _ in 0..num_trials {
        let hash_fn = family(random::<u64>(), num_buckets);
        let distribution = get_hash_value_distribution(&keys, num_buckets, hash_fn);
        let num_collisions = get_num_collisions(&distribution);
        total_num_collisions += num_collisions as u64;
    }

    let average_collisions = total_num_collisions as f64 / num_trials as f64;
    let total_pairs = keys.len() * (keys.len() - 1) / 2;
    let actual_prob = average_collisions / total_pairs as f64;

    (actual_prob, num_buckets)
}

/// Check the pairwise independence of two keys.
pub fn check_pairwise_independence<K>(
    key_a: &K,
    key_b: &K,
    family: &dyn Fn(u64, usize) -> Box<dyn Fn(&K) -> usize>,
    num_iters: usize,
) -> bool {
    let num_buckets = (num_iters as f64 / 5.0).sqrt().floor();
    let mut pairs: HashMap<(usize, usize), usize> = HashMap::new();
    let expected_prob = 1.0 / num_buckets.powi(2);

    for _ in 0..num_iters {
        let hash_fn = family(random::<u64>(), num_buckets as usize);
        let hash_a = hash_fn(key_a);
        let hash_b = hash_fn(key_b);
        *pairs.entry((hash_a, hash_b)).or_insert(0) += 1;
    }

    let actual_prob = pairs
        .iter()
        .fold(f64::NEG_INFINITY, |max_prob, (_, count)| {
            let prob = *count as f64 / num_iters as f64;
            if prob > max_prob {
                prob
            } else {
                max_prob
            }
        });

    actual_prob > expected_prob
}

#[derive(Debug)]
pub enum UniversalityGuarantee {
    _2Universal,
    Strong,
}

pub fn check_universality_guarantee<K>(
    guarantee: UniversalityGuarantee,
    num_trials: usize,
    family: &dyn Fn(u64, usize) -> Box<dyn Fn(&K) -> usize>,
    keys: &Vec<K>,
) where
    K: Hash + Eq + Clone,
{
    let (collision_prob, num_buckets) = get_collision_prob(keys, family, num_trials);

    if matches!(
        guarantee,
        UniversalityGuarantee::_2Universal | UniversalityGuarantee::Strong
    ) {
        let expected_prob = 1.0 / num_buckets as f64;
        assert!(
            collision_prob <= expected_prob,
            "The hash function does not satisfy the collision probability constraint:\n
            actual collision probability = {}\n
            expected collision probability = {}\n\
            number of buckets = {}.",
            collision_prob,
            expected_prob,
            num_buckets,
        );
    }

    if matches!(guarantee, UniversalityGuarantee::Strong) {
        for _ in 0..num_trials {
            let key_a = keys.choose(&mut thread_rng()).unwrap();
            let key_b = keys.choose(&mut thread_rng()).unwrap();
            let hash_value_independence = check_pairwise_independence(key_a, key_b, family, 99999);
            assert!(
                hash_value_independence,
                "The hash function does not satisfy the pairwise independence constraint."
            );
        }
    }
}
