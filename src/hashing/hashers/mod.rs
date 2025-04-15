//! [`crate::core::Hasher`] implementations.
//!
//! # Notes
//!
//! - Const-hasher implementations are supposed to be equivalent to the non-const ones
//!   but their `from_seed` methods are an exception - since they are using different PRNG.
//!   `from_state` method is the constructor that is supposed to be fully equivalent.
pub mod msp;
pub use msp::*;
