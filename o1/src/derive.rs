/// Derives an `o1_core::Hasher` implementation for structs and enums.
///
/// The macro examines the input type, generates a companion `Hasher` implementation, and honours
/// optional `#[o1(...)]` attributes:
///
/// - `#[o1(hasher = path::ToFamily)]` selects the hashing family for the whole type or for a
///   specific field.
/// - `#[o1(type_name = "CustomHasher")]` overrides the generated hasher’s type name.
/// - `#[o1(skip)]` excludes a field from hashing.
///
/// # Example
///
/// ```
/// use o1::hashing::hashers::msp::MSPHasher;
/// use o1_core::Hasher as _;
/// use o1_derive::Hasher;
///
/// #[derive(Hasher)]
/// #[o1(hasher = MSPHasher)]
/// struct Key {
///     id: u32,
///     #[o1(skip)]
///     memo: &'static str,
/// }
///
/// let hasher = KeyHasher::from_seed(13, 32);
/// let hash = hasher.hash(&Key { id: 7, memo: "hint" });
/// assert!(hash < 32);
/// ```
#[doc(no_inline)]
pub use o1_derive::Hasher;
