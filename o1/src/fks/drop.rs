//! Implements [`Drop`] for [`FKSMap`].
use crate::fks::FKSMap;
use bitvec::prelude::*;
use o1_core::Hasher;

/// Deinitializes only the initialized slots and skips the non-initialized ones.
impl<K: Eq, V, H: Hasher<K>> Drop for FKSMap<'_, K, V, H> {
    fn drop(&mut self) {
        debug_assert!(
            self.slots.is_borrowed() && self.buckets.is_borrowed()
                || self.slots.is_owned() && self.buckets.is_owned(),
            "FKSMap's memory allocation is inconsistent."
        );

        if self.slots.is_borrowed() && self.buckets.is_borrowed() {
            return;
        }
        for bucket in self.buckets.as_slice() {
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
