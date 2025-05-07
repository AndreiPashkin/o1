//! Implements [`HashMap`] for [`FKSMap`].

use crate::core::{HashMap, Hasher};
use crate::fks::FKSMap;
use bitvec::prelude::*;
use bitvec::view::BitView;
use std::borrow::Borrow;
use std::fmt::Debug;

impl<K: Eq + Debug, V, H: Hasher<K>> HashMap<K, V, H> for FKSMap<'_, K, V, H> {
    fn get<Q, QH>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: ?Sized + Eq,
        QH: for<'a> Hasher<&'a Q>,
        H: Borrow<QH>,
    {
        let hasher = self.l1_hasher.borrow();
        let bucket_idx = hasher.hash(key) as usize;
        let bucket = &self.buckets[bucket_idx];
        let data_idx: usize = match bucket.num_slots() {
            0 => return None,
            1 => bucket.offset,
            _ => {
                let hasher = bucket.hasher.borrow();
                let hash = hasher.hash(key);
                let is_set = unsafe {
                    bucket
                        .slots
                        .view_bits::<Lsb0>()
                        .get(hash as usize)
                        .unwrap_unchecked()
                };
                if !is_set {
                    return None;
                }
                bucket.offset + hash as usize
            }
        };

        let (k, v) = unsafe { &self.slots[data_idx].assume_init_ref() };

        if k.borrow() == key {
            Some(v)
        } else {
            None
        }
    }

    fn len(&self) -> usize {
        self.slots.len()
    }

    fn is_empty(&self) -> bool {
        self.slots.is_empty()
    }

    fn load_factor(&self) -> f64 {
        self.slots.len() as f64 / self.buckets.len() as f64
    }

    fn num_collisions(&self) -> usize {
        self.buckets
            .iter()
            .map(|b| {
                if b.num_slots() > 1 {
                    b.num_slots() - 1
                } else {
                    0
                }
            })
            .sum()
    }
}
