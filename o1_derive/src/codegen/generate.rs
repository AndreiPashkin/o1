//! Provides the main code generation method of the [`Codegen`] type.
use super::Codegen;
use crate::error::O1DeriveError;
use crate::ir::DeriveData;
use proc_macro2::{Ident, TokenStream};
use quote::quote;

impl Codegen {
    /// Generates the complete token stream for the derived hasher and its state.
    ///
    /// The entrypoint of the [`Codegen`] for the external users.
    pub fn generate(ir: &DeriveData) -> Result<TokenStream, O1DeriveError> {
        let hasher_name = Self::make_hasher_name(ir);
        let hasher_state_name = Self::make_hasher_state_name(&hasher_name, ir);

        let target_name: Ident = ir.type_name().clone();

        let state_definition = Self::generate_state(ir, &hasher_state_name)?;
        let hasher_definition = Self::generate_hasher_struct(&hasher_name, &hasher_state_name)?;
        let const_impl =
            Self::generate_const_impl(ir, &hasher_name, &hasher_state_name, &target_name)?;
        let runtime_impl =
            Self::generate_runtime_impl(ir, &hasher_name, &hasher_state_name, &target_name)?;

        let expanded = quote! {
            #state_definition
            #hasher_definition
            #const_impl
            #runtime_impl
        };

        Ok(expanded)
    }
}
