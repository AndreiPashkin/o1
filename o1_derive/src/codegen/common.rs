//! Shared helper-methods.
use super::Codegen;
use crate::error::O1DeriveError;
use crate::ir::{DeriveData, FieldData, VariantData};
use proc_macro2::{Ident, TokenStream};
use quote::quote;

impl Codegen {
    /// Makes the identifier for the top-level hasher type.
    pub(super) fn make_hasher_name(ir: &DeriveData) -> Ident {
        if let Some(name_override) = ir.type_name_override().cloned() {
            name_override
        } else {
            let type_name = ir.type_name().to_string();
            let hasher_name = format!("{}Hasher", type_name);
            Ident::new(&hasher_name, ir.span())
        }
    }

    /// Derives the identifier for the hasher state struct from the hasher name.
    pub(super) fn make_hasher_state_name(hasher_name: &Ident, ir: &DeriveData) -> Ident {
        let state_name = format!("{}State", hasher_name);
        Ident::new(&state_name, ir.span())
    }

    /// Produces a unique identifier for the stored state of a single field hasher.
    pub(super) fn make_field_state_name(
        ir: &DeriveData,
        variant: &VariantData,
        field: &FieldData,
    ) -> Ident {
        let prefix = if ir.is_enum() {
            let variant_name = variant.name().to_string();
            let lowered = variant_name.to_lowercase();
            format!("variant{}_{}", variant.index(), lowered)
        } else {
            String::new()
        };

        let base_name = match field.name() {
            Some(name) => name.to_string(),
            None => format!("field_{}", field.index()),
        };

        let full = if prefix.is_empty() {
            format!("{}_hasher", base_name)
        } else {
            format!("{}_{}_hasher", prefix, base_name)
        };

        Ident::new(&full, field.span())
    }

    /// Creates a consistent internal error for missing hasher-family annotations.
    pub(super) fn missing_hasher_path_error(ir: &DeriveData) -> O1DeriveError {
        O1DeriveError::internal_error("Missing hasher path in IR", ir.span())
    }

    /// Generates initialization code for all field hashers and the combiner hasher.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn make_field_initializers(
        ir: &DeriveData,
        hasher_path: &syn::Path,
        num_hash_components: usize,
        num_buckets_name: &Ident,
        rng_name: &Ident,
        is_const: bool,
    ) -> Result<(Vec<TokenStream>, Vec<Ident>, TokenStream), O1DeriveError> {
        let mut field_inits: Vec<TokenStream> = Vec::new();
        let mut field_names: Vec<Ident> = Vec::new();

        for variant in ir.variants() {
            for field in variant.non_skipped_fields() {
                let name = Self::make_field_state_name(ir, variant, field);
                let field_type = field.type_().clone();
                let family = field.hasher_path().unwrap_or(hasher_path);
                let init_tokens = if is_const {
                    quote! {
                        let #name = #family::<#field_type>::from_seed_const(#rng_name.next(), #num_buckets_name);
                    }
                } else {
                    quote! {
                        let #name = <#family<#field_type> as o1_core::core::Hasher<#field_type>>::from_seed(#rng_name.next(), #num_buckets_name);
                    }
                };
                field_inits.push(init_tokens);
                field_names.push(name);
            }
        }

        let combiner_init = if is_const {
            quote! {
                let combiner_hasher = #hasher_path::<[u32; #num_hash_components]>::from_seed_const(#rng_name.next(), #num_buckets_name);
            }
        } else {
            quote! {
                let combiner_hasher = <#hasher_path<[u32; #num_hash_components]> as o1_core::core::Hasher<[u32; #num_hash_components]>>::from_seed(#rng_name.next(), #num_buckets_name);
            }
        };

        Ok((field_inits, field_names, combiner_init))
    }
}
