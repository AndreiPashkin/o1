//! Implements generation of the state type.
use super::Codegen;
use crate::error::O1DeriveError;
use crate::ir::DeriveData;
use proc_macro2::{Ident, TokenStream};
use quote::quote;

impl Codegen {
    /// Emits the state struct and its conditional `Copy` implementation.
    ///
    /// # Notes
    ///
    /// - `Copy` implementation is conditioned on whether all field hashers are `Copy`.
    pub(super) fn generate_state(
        ir: &DeriveData,
        hasher_state_name: &Ident,
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

        let mut field_state_decls: Vec<TokenStream> = Vec::new();
        let mut copy_bounds: Vec<TokenStream> = Vec::new();
        for variant in ir.variants() {
            for field in variant.non_skipped_fields() {
                let field_name = Self::make_field_state_name(ir, variant, field);
                let field_type = field.type_().clone();
                let chosen_family = field.hasher_path().unwrap_or(hasher_path);
                let decl = quote! {
                    #field_name: #chosen_family<#field_type>,
                };
                field_state_decls.push(decl);

                copy_bounds.push(quote! {
                    #chosen_family<#field_type>: Copy
                });
            }
        }

        let combiner_decl = quote! {
            combiner_hasher: #hasher_path<[u32; #num_hash_components]>,
        };

        copy_bounds.push(quote! {
            #hasher_path<[u32; #num_hash_components]>: Copy
        });

        let state = quote! {
            #[derive(Clone, Default)]
            pub struct #hasher_state_name {
                num_buckets: u32,
                #combiner_decl
                #( #field_state_decls )*
            }
        };
        let copy_impl = quote! {
            impl Copy for #hasher_state_name where
                #( #copy_bounds ),*
            {}
        };

        Ok(quote! {
            #state
            #copy_impl
        })
    }
}
