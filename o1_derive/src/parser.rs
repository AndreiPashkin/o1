/// Implements the parsing layer, which converts the raw input AST into the IR.
use crate::error::O1DeriveError;
use crate::ir::{DeriveData, DeriveKind, FieldData, VariantData};
use proc_macro2::{Ident, Span};
use quote::ToTokens as _;
use syn::spanned::Spanned as _;
use syn::{Attribute, Data, DeriveInput, LitStr, Path};
use synstructure::Structure;

/// Encapsulates all the parsing logic.
pub struct Parser;

/// Parsed attributes for the whole type.
#[derive(Default, Debug, Clone)]
struct TypeAttributes {
    hasher_path: Option<Path>,
    type_name_override: Option<Ident>,
}

/// Parsed attributes for a single field.
#[derive(Default, Debug, Clone)]
struct FieldAttributes {
    hasher_path: Option<Path>,
    skip: bool,
}

impl Parser {
    /// Parse input AST into IR `DeriveData`.
    pub fn parse(input: &DeriveInput) -> Result<DeriveData, O1DeriveError> {
        let span: Span = input.ident.span();
        let type_identifier: Ident = input.ident.clone();
        let kind = Self::parse_derive_kind(input)?;

        let type_attributes = Self::parse_type_attributes(&input.attrs)?;
        let hasher_path = type_attributes.hasher_path.clone();
        if hasher_path.is_none() {
            return Err(O1DeriveError::parse_error(
                "Missing required `#[o1(hasher = <path>)]` attribute on the type.",
                span,
            ));
        }
        let type_name_override = type_attributes.type_name_override.clone();

        let variants = Self::parse_variants(input, &kind, &type_identifier, span)?;
        let max_fields = Self::max_non_skipped_fields(&variants);

        Ok(DeriveData::new(
            type_identifier,
            type_name_override,
            kind,
            input.clone(),
            hasher_path,
            variants,
            max_fields,
            span,
        ))
    }

    /// Determines whether the input is a struct or an enum.
    fn parse_derive_kind(input: &DeriveInput) -> Result<DeriveKind, O1DeriveError> {
        Ok(match &input.data {
            Data::Struct(_) => DeriveKind::Struct,
            Data::Enum(_) => DeriveKind::Enum,
            Data::Union(u) => {
                return Err(O1DeriveError::parse_error(
                    "Unions are not supported.",
                    u.union_token.span(),
                ))
            }
        })
    }

    /// Collect variants and fields using `synstructure` uniformly for structs and enums.
    fn parse_variants(
        input: &DeriveInput,
        kind: &DeriveKind,
        type_name: &Ident,
        span: Span,
    ) -> Result<Vec<VariantData>, O1DeriveError> {
        let structure = Structure::new(input);
        let mut variants_data: Vec<VariantData> = Vec::new();
        for (variant_index, variant) in structure.variants().iter().enumerate() {
            let mut fields = Vec::new();
            for (field_index, binding) in variant.bindings().iter().enumerate() {
                let field = binding.ast();
                let field_attributes = Self::parse_field_attributes(&field.attrs)?;
                let optional_field_name = field.ident.clone();
                let field_type = field.ty.clone();
                let field_span = optional_field_name
                    .as_ref()
                    .map(|id| id.span())
                    .unwrap_or_else(|| field_type.span());
                fields.push(FieldData::new(
                    field_index,
                    optional_field_name,
                    field_type,
                    field_attributes.skip,
                    field_attributes.hasher_path,
                    field_span,
                ));
            }

            let (variant_name, _variant_span) = match kind {
                DeriveKind::Struct => (type_name.clone(), span),
                DeriveKind::Enum => (variant.ast().ident.clone(), variant.ast().ident.span()),
            };

            variants_data.push(VariantData::new(variant_index, variant_name, fields));
        }
        Ok(variants_data)
    }

    /// Return the maximum number of non-skipped fields across all variants.
    fn max_non_skipped_fields(variants: &[VariantData]) -> u32 {
        variants
            .iter()
            .map(|v| v.non_skipped_fields().count() as u32)
            .max()
            .unwrap_or(0)
    }
    fn parse_type_attributes(attrs: &[Attribute]) -> Result<TypeAttributes, O1DeriveError> {
        let mut result = TypeAttributes::default();
        for attr in attrs.iter().filter(|a| a.path().is_ident("o1")) {
            attr.parse_nested_meta(|meta| {
                let key = meta.path.get_ident().map(|i| i.to_string());
                match key.as_deref() {
                    Some("hasher") => {
                        let value: Path = meta.value()?.parse()?;
                        result.hasher_path = Some(value);
                        Ok(())
                    }
                    Some("type_name") => {
                        let value: LitStr = meta.value()?.parse()?;
                        let ident = syn::parse_str::<Ident>(&value.value())?;
                        result.type_name_override = Some(ident);
                        Ok(())
                    }
                    _ => Err(syn::Error::new(
                        meta.path.span(),
                        format!(
                            "Unexpected metadata key `{}` in type-level #[o1(...)].",
                            meta.path.to_token_stream()
                        ),
                    )),
                }
            })
            .map_err(O1DeriveError::from)?;
        }
        Ok(result)
    }

    fn parse_field_attributes(attrs: &[Attribute]) -> Result<FieldAttributes, O1DeriveError> {
        let mut result = FieldAttributes::default();
        for attr in attrs.iter().filter(|a| a.path().is_ident("o1")) {
            attr.parse_nested_meta(|meta| {
                let key = meta.path.get_ident().map(|i| i.to_string());
                match key.as_deref() {
                    Some("skip") => {
                        result.skip = true;
                        Ok(())
                    }
                    Some("hasher") => {
                        let value: Path = meta.value()?.parse()?;
                        result.hasher_path = Some(value);
                        Ok(())
                    }
                    _ => Err(syn::Error::new(
                        meta.path.span(),
                        format!(
                            "Unexpected metadata key `{}` in field-level #[o1(...)].",
                            meta.path.to_token_stream()
                        ),
                    )),
                }
            })
            .map_err(O1DeriveError::from)?;
        }
        if result.skip {
            result.hasher_path = None;
        }
        Ok(result)
    }
}
