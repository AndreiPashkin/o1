//! A procedural macro to derive the `Hasher` trait for structs and enums.
mod codegen;
mod error;
mod ir;
mod parser;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

use crate::codegen::Codegen;
use crate::error::O1DeriveError;
use crate::parser::Parser;

/// Derives an `o1_core::Hasher` implementation for structs and enums.
///
/// The macro examines the input type, generates a companion `Hasher` implementation, and honours
/// optional `#[o1(...)]` attributes:
///
/// - `#[o1(hasher = path::ToFamily)]` selects the hashing family for the whole type or for a
///   specific field.
/// - `#[o1(type_name = "CustomHasher")]` overrides the generated hasher’s type name.
/// - `#[o1(skip)]` excludes a field from hashing.
#[proc_macro_derive(Hasher, attributes(o1))]
pub fn derive_hasher(input: TokenStream) -> TokenStream {
    let input_ast = parse_macro_input!(input as DeriveInput);

    match Parser::parse(&input_ast) {
        Ok(ir) => match Codegen::generate(&ir) {
            Ok(tokens) => tokens.into(),
            Err(err) => {
                let syn_err: syn::Error = <O1DeriveError as Into<syn::Error>>::into(err);
                syn_err.to_compile_error().into()
            }
        },
        Err(err) => {
            let syn_err: syn::Error = <O1DeriveError as Into<syn::Error>>::into(err);
            syn_err.to_compile_error().into()
        }
    }
}
