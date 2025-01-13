/// Extract the top `num_bits` bits from a 64-bit value.
///
/// Useful as a faster alternative to the modulo operation of this kind: `value % (2 ** num_bits)`.
#[inline]
pub const fn extract_bits_32(value: u64, num_bits: u32) -> u32 {
    debug_assert!(num_bits <= 32, r#""num_bits" must be <= 32"#);

    (value >> (64 - num_bits)) as u32
}

/// Performs a modulo operation by a Mersenne prime.
///
/// Faster equivalent of the operation: `x % p`, where `p == 2 ** n`.
#[inline]
pub const fn mod_mersenne_prime(x: u64, p: u64, n: u32) -> u64 {
    debug_assert!(
        p == (2_u64.pow(n) - 1),
        r#""p" must be a Mersenne prime, so "p == 2 ** s - 1" constraint should stand."#
    );

    let result = (x & p) + (x >> n);
    if result >= p {
        result - p
    } else {
        result
    }
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
