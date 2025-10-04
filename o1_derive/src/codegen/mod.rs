//! Implementation of the code-generation layer of the macro.
//!
//! Provides [`Codegen`] - an entrypoint into code-generation layer.
mod common;
mod const_impl;
mod core;
mod generate;
mod hash_match;
mod hasher_struct;
mod runtime_impl;
mod state;
pub use core::*;
