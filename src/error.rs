//! Error definitions.
use thiserror::Error;

/// Project-wise error type.
#[derive(Error, Debug)]
pub enum O1Error {
    /// Might occur during construction of a hash table and means failure to find a hash function
    /// that resolves in the context determined by the hashing scheme.
    #[error("Unable to find hash function suitable for resolving collisions.")]
    UnableToFindHashFunction,
}
