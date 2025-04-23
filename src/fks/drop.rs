//! Implements [`Drop`] for [`FKSMap`].
use crate::core::Hasher;
use crate::fks::FKSMap;
use bitvec::prelude::*;

/// Deinitializes only the initialized slots and skips the non-initialized ones.
impl<K: Eq, V, H: Hasher<K>> Drop for FKSMap<K, V, H> {
    fn drop(&mut self) {
        for bucket in &self.buckets {
            if bucket.num_slots() == 0 {
                continue;
            }

            for slot_idx in bucket.slots.view_bits::<Lsb0>().iter_ones() {
                let data_idx = bucket.offset + slot_idx;
                unsafe { self.slots[data_idx].assume_init_drop() };
            }
        }
    }
}
