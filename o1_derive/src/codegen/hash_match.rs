//! Provides the hashing-logic generating method.
use super::Codegen;
use crate::error::O1DeriveError;
use crate::ir::DeriveData;
use proc_macro2::TokenStream;
use quote::quote;
use synstructure::BindStyle;

impl Codegen {
    /// Builds the body of the hashing logic:
    ///
    ///   - Delegate to hasher of each field.
    ///   - Combine the resulting hashes into the final value using a dedicated combiner-hasher.
    ///
    /// # Notes
    ///
    /// - Generation of match-arms is driven by the `synstructure`.
    pub(super) fn generate_hash_match(
        ir: &DeriveData,
        family: &syn::Path,
        num_hash_components: usize,
        is_const: bool,
    ) -> Result<TokenStream, O1DeriveError> {
        let mut structure = ir.structure();
        structure.bind_with(|_| BindStyle::Ref);

        let mut arms: Vec<TokenStream> = Vec::new();
        for (variant_idx, variant_info) in structure.variants().iter().enumerate() {
            let ir_variant = ir
                .get_variant(variant_idx)
                .ok_or_else(|| O1DeriveError::internal_error("IR variant missing", ir.span()))?;

            let mut comps: Vec<TokenStream> = Vec::new();
            if ir.is_enum() {
                comps.push(quote! { #variant_idx as u32 });
            }

            for (i, binding) in variant_info.bindings().iter().enumerate() {
                let field = ir_variant
                    .get_field(i)
                    .ok_or_else(|| O1DeriveError::internal_error("IR field missing", ir.span()))?;
                if field.skip() {
                    continue;
                }
                let state_name = Self::make_field_state_name(ir, ir_variant, field);
                let bident = binding.binding.clone();
                let expr = if is_const {
                    quote! { self.state.#state_name.hash_const(#bident) }
                } else {
                    quote! { self.state.#state_name.hash(#bident) }
                };
                comps.push(expr);
            }

            let current = comps.len();
            if current < num_hash_components {
                let pad = num_hash_components - current;
                for _ in 0..pad {
                    comps.push(quote! { 0u32 });
                }
            }
            let comps_array = quote! { [ #( #comps ),* ] };

            let combine = if is_const {
                quote! {
                    let __components: [u32; #num_hash_components] = #comps_array;
                    self.state.combiner_hasher.hash_const(&__components)
                }
            } else {
                quote! {
                    let __components: [u32; #num_hash_components] = #comps_array;
                    <#family<[u32; #num_hash_components]> as o1_core::core::Hasher<[u32; #num_hash_components]>>::hash(&self.state.combiner_hasher, &__components)
                }
            };

            let pat = variant_info.pat();
            arms.push(quote! { #pat => { #combine } });
        }

        Ok(quote! { match value { #( #arms ),* } })
    }
}
