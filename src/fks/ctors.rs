//! Implements constructors for [`FKSMap`].
use crate::core::Hasher;
use crate::error::O1Error;
use crate::error::O1Error::UnableToFindHashFunction;
use crate::fks::{Bucket, FKSMap};
use bitvec::prelude::*;
use rand::{RngCore, SeedableRng};
use rand_xoshiro::Xoshiro256PlusPlus;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::mem::MaybeUninit;

impl<K: Eq + Debug, V, H: Hasher<K>> FKSMap<'_, K, V, H> {
    const MAX_KEYS_PER_BUCKET: u32 = 5;

    /// Attempts to find the L1 hash function.
    ///
    /// # Parameters
    ///
    /// - `rng`: A random number generator.
    /// - `load_factor`: The desirable load factor.
    /// - `num_trials`: The maximum number of trials to find the hash function.
    /// - `data`: The data to be hashed.
    fn try_resolve_l1(
        rng: &mut Xoshiro256PlusPlus,
        load_factor: f32,
        num_trials: usize,
        data: &[(K, V)],
    ) -> Result<(H, Vec<BitVec>), O1Error> {
        for _ in 0..num_trials {
            let l1_hasher = H::from_seed(
                rng.next_u64(),
                // TODO: Is this correct? Shouldn't the number of buckets be lower than the size of
                //       the data?
                (data.len() as f32 / load_factor).ceil() as u32,
            );
            let num_buckets = l1_hasher.num_buckets();

            let mut buckets = Vec::<MaybeUninit<Bucket<K, H>>>::with_capacity(num_buckets as usize);

            unsafe {
                buckets.set_len(num_buckets as usize);
            }

            let mut bucket_to_keys = vec![bitvec![0; data.len()]; num_buckets as usize];

            let mut max_keys_per_bucket: u64 = 0;

            bucket_to_keys.iter_mut().for_each(|v| v.fill(false));

            for (i, (k, _)) in data.iter().enumerate() {
                let hash = l1_hasher.hash(k);
                bucket_to_keys[hash as usize].set(i, true);
            }

            for i in 0..num_buckets {
                let keys_in_bucket = bucket_to_keys[i as usize].count_ones();
                max_keys_per_bucket = max_keys_per_bucket.max(keys_in_bucket as u64);
            }

            if max_keys_per_bucket <= Self::MAX_KEYS_PER_BUCKET as u64 {
                return Ok((l1_hasher, bucket_to_keys));
            }
        }
        Err(UnableToFindHashFunction)
    }

    /// Attempt to find the L2 hash function for the given bucket.
    ///
    /// # Parameters
    ///
    /// - `rng`: A random number generator.
    /// - `bucket_idx`: The global index of the bucket in the L1 table.
    /// - `current_offset`: The current global offset of the bucket.
    /// - `data`: The complete input data.
    /// - `bucket_to_keys`: A vec of bit-masks where each bit-mask is a bucket
    ///                     and each bit is a key in the input data.
    /// - `num_trials`: The maximum number of trials to find the hash function.
    fn try_resolve_bucket(
        rng: &mut Xoshiro256PlusPlus,
        bucket_idx: usize,
        current_offset: usize,
        data: &[(K, V)],
        bucket_to_keys: &[BitVec],
        num_trials: usize,
    ) -> Result<Bucket<K, H>, O1Error> {
        for _ in 0..num_trials {
            let keys = &bucket_to_keys[bucket_idx];
            let num_keys: usize = keys.count_ones();
            if num_keys == 0 {
                // Unoccupied bucket
                return Ok(Bucket::default());
            }

            let hasher = H::from_seed(rng.next_u64(), num_keys as u32);
            let num_slots = hasher.num_buckets();

            let mut slots: u8 = 0;

            for key_idx in keys.iter_ones() {
                let key = &data[key_idx].0;
                let hash = hasher.hash(key);
                slots.view_bits_mut::<Lsb0>().set(hash as usize, true);
            }

            if slots.count_ones() == num_keys as u32 {
                return Ok(Bucket {
                    offset: current_offset,
                    slots,
                    num_slots: num_slots as u8,
                    hasher,
                    key_type: PhantomData,
                });
            }
        }

        Err(UnableToFindHashFunction)
    }

    /// Fills the hash table with data based on selected L1 and L2 hash functions.
    fn fill_slots(
        data: Box<[(K, V)]>,
        buckets: &[Bucket<K, H>],
        slots: &mut [MaybeUninit<(K, V)>],
        l1_hasher: &H,
    ) {
        let mut max_data_idx: usize = 0;
        for (k, v) in data.into_vec().into_iter() {
            let bucket_idx = l1_hasher.hash(&k) as usize;
            let bucket: &Bucket<_, _> = &buckets[bucket_idx];
            let data_idx = bucket.hasher.hash(&k) as usize + bucket.offset;
            slots[data_idx] = MaybeUninit::<(K, V)>::new((k, v));
            max_data_idx = data_idx.max(max_data_idx);
        }
    }

    const MAX_L1_TRIALS: usize = 999;
    const MAX_L2_TRIALS: usize = 999;

    /// Creates a new [`FKSMap`] with the given data, seed, and minimum load factor.
    ///
    /// # Parameters
    ///
    /// - `data`: The data to be hashed.
    /// - `seed`: The seed for the random number generator.
    /// - `min_load_factor`: The minimum load factor.
    pub fn new(data: Box<[(K, V)]>, seed: u64, min_load_factor: f32) -> Result<Self, O1Error> {
        debug_assert!(min_load_factor > 0.0 && min_load_factor <= 1.0);

        let mut rng = Xoshiro256PlusPlus::seed_from_u64(seed);

        let mut load_factor = 1.0;

        let l1_hasher: H;
        let bucket_to_keys: Vec<BitVec>;

        // Try to resolve the level-1 gradually lowering the load factor after each failure.
        loop {
            if let Ok(l1_result) =
                Self::try_resolve_l1(&mut rng, load_factor, Self::MAX_L1_TRIALS, &data)
            {
                l1_hasher = l1_result.0;
                bucket_to_keys = l1_result.1;
                break;
            }
            load_factor -= 0.05;

            if load_factor < min_load_factor {
                return Err(UnableToFindHashFunction);
            }
        }

        let l1_num_buckets: u32 = l1_hasher.num_buckets();
        let mut buckets = Vec::<Bucket<K, H>>::with_capacity(l1_num_buckets as usize);

        let mut current_offset: usize = 0;

        for bucket_idx in 0..l1_num_buckets {
            let resolved_bucket = Self::try_resolve_bucket(
                &mut rng,
                bucket_idx as usize,
                current_offset,
                &data,
                &bucket_to_keys,
                Self::MAX_L2_TRIALS,
            )?;

            current_offset += resolved_bucket.num_slots();
            buckets.push(resolved_bucket);
        }

        let mut slots = Vec::<MaybeUninit<(K, V)>>::with_capacity(current_offset);
        unsafe { slots.set_len(slots.capacity()) };

        Self::fill_slots(data, &buckets, &mut slots, &l1_hasher);

        Ok(Self {
            l1_hasher,
            buckets: buckets.into(),
            slots: slots.into(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::HashMap;
    use crate::fks::FKSMap;
    use crate::generate_map_tests;
    use std::fmt::Debug;

    fn factory<'a, K: Eq + Debug, V: Copy + Debug, H: Hasher<K>>(
        data: Box<[(K, V)]>,
    ) -> FKSMap<'a, K, V, H> {
        FKSMap::new(data, 0, 0.75).unwrap()
    }

    generate_map_tests!(FKSMap, factory);
}
