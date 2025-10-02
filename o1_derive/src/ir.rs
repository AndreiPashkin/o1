/// Intermediate representation layer that holds information extracted from the raw input important
/// for code generation.
///
/// # Notes
///
/// - IR doesn't define separate types for structs. Instead, structs are treated as a single-variant
///   enum and both structs and enum variants are represented by [`VariantData`].
use proc_macro2::{Ident, Span};
use syn::{DeriveInput, Path, Type};
use synstructure::Structure;

/// Whether the input is a struct or an enum.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DeriveKind {
    Struct,
    Enum,
}

/// Complete IR for a single derive-macro invocation
///
/// The top-level node of the IR.
#[derive(Clone, Debug)]
pub struct DeriveData {
    type_name: Ident,
    type_name_override: Option<Ident>,
    kind: DeriveKind,
    input: DeriveInput,
    hasher_path: Option<Path>,
    variants: Vec<VariantData>,
    max_fields: u32,
    span: Span,
}

impl DeriveData {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        type_name: Ident,
        type_name_override: Option<Ident>,
        kind: DeriveKind,
        input: DeriveInput,
        hasher_path: Option<Path>,
        variants: Vec<VariantData>,
        max_fields: u32,
        span: Span,
    ) -> Self {
        Self {
            type_name,
            type_name_override,
            kind,
            input,
            hasher_path,
            variants,
            max_fields,
            span,
        }
    }
    pub fn type_name(&self) -> &Ident {
        &self.type_name
    }
    pub fn type_name_override(&self) -> Option<&Ident> {
        self.type_name_override.as_ref()
    }
    pub fn is_enum(&self) -> bool {
        self.kind == DeriveKind::Enum
    }
    pub fn hasher_path(&self) -> Option<&Path> {
        self.hasher_path.as_ref()
    }
    pub fn variants(&self) -> &[VariantData] {
        &self.variants
    }
    /// Get the maximum possible number of fields in any variant of the enum or the struct.
    pub fn max_fields(&self) -> u32 {
        self.max_fields
    }
    /// Get a variant by the index.
    pub fn get_variant(&self, index: usize) -> Option<&VariantData> {
        self.variants.get(index)
    }
    pub fn span(&self) -> Span {
        self.span
    }
    pub fn structure(&self) -> Structure<'_> {
        Structure::new(&self.input)
    }
}

/// Represents a single variant of an enum or the whole struct.
#[derive(Clone, Debug)]
pub struct VariantData {
    index: usize,
    name: Ident,
    fields: Vec<FieldData>,
}

impl VariantData {
    pub fn new(index: usize, name: Ident, fields: Vec<FieldData>) -> Self {
        Self {
            index,
            name,
            fields,
        }
    }
    pub fn index(&self) -> usize {
        self.index
    }
    pub fn name(&self) -> &Ident {
        &self.name
    }
    pub fn non_skipped_fields(&self) -> impl Iterator<Item = &FieldData> {
        self.fields.iter().filter(|f| !f.skip())
    }
    /// Get a field by its index.
    pub fn get_field(&self, index: usize) -> Option<&FieldData> {
        self.fields.get(index)
    }
}

/// A single field of a struct or an enum variant.
#[derive(Clone, Debug)]
pub struct FieldData {
    index: usize,
    name: Option<Ident>,
    type_: Type,
    skip: bool,
    hasher_path: Option<Path>,
    span: Span,
}

impl FieldData {
    pub fn new(
        index: usize,
        name: Option<Ident>,
        type_: Type,
        skip: bool,
        hasher_path: Option<Path>,
        span: Span,
    ) -> Self {
        Self {
            index,
            name,
            type_,
            skip,
            hasher_path,
            span,
        }
    }
    pub fn index(&self) -> usize {
        self.index
    }
    pub fn name(&self) -> Option<&Ident> {
        self.name.as_ref()
    }
    pub fn type_(&self) -> &Type {
        &self.type_
    }
    pub fn skip(&self) -> bool {
        self.skip
    }
    pub fn hasher_path(&self) -> Option<&Path> {
        self.hasher_path.as_ref()
    }
    pub fn span(&self) -> Span {
        self.span
    }
}
