//! Implements a general purpose hasher that provides strong universal guarantees and is based
//! on multiply-shift and polynomial hash function families (hence MSP).
mod core;
pub use core::*;
mod smallint;
pub use smallint::*;
mod int64;
pub use int64::*;
mod bigint;
pub use bigint::*;
mod string;
pub use string::*;
mod option;
pub use option::*;
