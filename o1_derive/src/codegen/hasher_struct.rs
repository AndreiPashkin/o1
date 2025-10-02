//! Declares the concrete hasher wrapper that stores state and implements traits.
use super::Codegen;
use crate::error::O1DeriveError;
use proc_macro2::{Ident, TokenStream};
use quote::quote;

impl Codegen {
    /// Emits the hasher type along with associated trait impls.
    pub(super) fn generate_hasher_struct(
        hasher_name: &Ident,
        hasher_state_name: &Ident,
    ) -> Result<TokenStream, O1DeriveError> {
        let tokens = quote! {
            pub struct #hasher_name<T> {
                state: #hasher_state_name,
                _phantom: core::marker::PhantomData<T>,
            }

            impl<T> Clone for #hasher_name<T> {
                fn clone(&self) -> Self {
                    Self {
                        state: self.state.clone(),
                        _phantom: core::marker::PhantomData,
                    }
                }
            }

            impl<T> core::default::Default for #hasher_name<T> {
                fn default() -> Self {
                    Self {
                        state: #hasher_state_name::default(),
                        _phantom: core::marker::PhantomData,
                    }
                }
            }

            impl<T> Copy for #hasher_name<T>
            where
                #hasher_state_name: Copy,
            {}
        };
        Ok(tokens)
    }
}
