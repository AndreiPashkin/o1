/// Generates standard test cases for hashers.
///
/// This macro generates test functions that verify:
/// - Equivalence between runtime and const-time methods
///
/// # Parameters
///
/// - `test_name`: The name of the test function
/// - `hasher_type`: The hasher type to test (e.g., `MSPHasher<u32>`)
/// - `key_type`: The key type to test (e.g., `u32`)
/// - `generate_key`: A function that generates a key value for testing
///
/// # Example
///
/// ```ignore
/// generate_hasher_tests!(
///     MSPHasher<u32>,
///     u32,
///     |rng| rng.random::<u32>()
/// );
/// ```
#[macro_export]
macro_rules! generate_hasher_tests {
    ($hasher_type:ty, $key_type:ty, $generate_key:expr$(,)?) => {
        compose_idents::compose_idents!(
            test_fn = concat(test_hasher_const_hashing_equivalence_, normalize($key_type)),
            {
                #[test]
                fn test_fn() {
                    use rand::SeedableRng;
                    use rand_chacha::ChaCha20Rng;

                    $crate::hasher_equivalence!(
                        $hasher_type,
                        $key_type,
                        &mut ChaCha20Rng::from_os_rng(),
                        $generate_key,
                        1 << 16,
                        50
                    );
                }
            }
        );
    };
}
pub use generate_hasher_tests;
