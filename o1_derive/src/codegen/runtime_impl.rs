//! Generates the runtime `Hasher` trait implementation for the derived type.
use super::Codegen;
use crate::error::O1DeriveError;
use crate::ir::DeriveData;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};

impl Codegen {
    /// Emits the implementation of [`o1_core::Hasher`] that drives runtime hashing.
    pub(super) fn generate_runtime_impl(
        ir: &DeriveData,
        hasher_name: &Ident,
        hasher_state_name: &Ident,
        target_name: &Ident,
    ) -> Result<TokenStream, O1DeriveError> {
        let max_fields = ir.max_fields() as usize;
        let num_hash_components: usize = if ir.is_enum() {
            max_fields + 1
        } else {
            max_fields
        };
        let hasher_path = ir
            .hasher_path()
            .ok_or_else(|| Self::missing_hasher_path_error(ir))?;

        let num_buckets_name = format_ident!("num_buckets");
        let rng_name = format_ident!("rng");
        let (field_inits, field_names, combiner_init) = Self::make_field_initializers(
            ir,
            hasher_path,
            num_hash_components,
            &num_buckets_name,
            &rng_name,
            false,
        )?;

        let hash_body = Self::generate_hash_match(ir, hasher_path, num_hash_components, false)?;
        let tokens = quote! {
            impl o1_core::core::Hasher<#target_name> for #hasher_name<#target_name> {
                type State = #hasher_state_name;

                fn make_state(seed: u64, num_buckets: u32) -> Self::State {
                    let seed = if seed == 0 { 1 } else { seed };
                    let mut #rng_name = o1::utils::xorshift::XorShift::<u64>::new(seed);
                    #combiner_init
                    #( #field_inits )*
                    #hasher_state_name {
                        num_buckets: combiner_hasher.num_buckets(),
                        combiner_hasher,
                        #( #field_names ),*
                    }
                }

                fn from_seed(seed: u64, num_buckets: u32) -> Self {
                    let state = Self::make_state(seed, num_buckets);
                    Self {
                        state,
                        _phantom: core::marker::PhantomData,
                    }
                }

                fn from_state(state: Self::State) -> Self {
                    Self {
                        state,
                        _phantom: core::marker::PhantomData,
                    }
                }

                fn state(&self) -> &Self::State { &self.state }

                fn num_buckets(&self) -> u32 { self.state.num_buckets }

                fn hash(&self, value: &#target_name) -> u32 { #hash_body }
            }
        };
        Ok(tokens)
    }
}
