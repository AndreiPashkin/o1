//! The implementation of the FKS perfect hashing approach [(Fredman et al., 1984)].
//!
//! [(Fredman et al., 1984)]: https://dl.acm.org/doi/10.1145/828.1884
mod core;
pub use core::*;
mod ctors;
mod drop;
mod hash_map;
