/// Extract the top `num_bits` bits from a 64-bit value.
///
/// Useful as a faster alternative to the modulo operation of this kind: `value % (2 ** num_bits)`.
#[inline]
pub const fn extract_bits_64<const SOURCE_BITS: u32>(value: u64, num_bits: u32) -> u32 {
    debug_assert!(num_bits <= 32, r#""num_bits" must be <= 32"#);

    (value >> (SOURCE_BITS - num_bits)) as u32
}

#[inline]
pub const fn extract_bits_128<const SOURCE_BITS: u32>(value: u128, num_bits: u32) -> u32 {
    debug_assert!(num_bits <= 32, r#""num_bits" must be <= 32"#);

    (value >> (SOURCE_BITS - num_bits)) as u32
}

/// Calculate the number of bits required to represent a given number of buckets.
#[allow(dead_code)]
pub const fn num_bits_for_buckets(num_buckets: u32) -> u32 {
    match num_buckets {
        0 => 0,
        1 => 1,
        _ => num_buckets.next_power_of_two().ilog2(),
    }
}

/// Calculate the number of bits required to represent a given number of buckets.
pub const fn num_buckets_for_bits(num_bits: u32) -> u32 {
    1 << num_bits
}
