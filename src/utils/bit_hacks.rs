/// Performs a modulo operation by a Mersenne prime.
///
/// Faster equivalent of the operation: `x % p`, where `p == 2 ** n`.
#[inline]
pub const fn mod_mersenne_prime<const P_E: u32, const P: u128>(x: u128) -> u128 {
    debug_assert!(
        P == (2_u128.pow(P_E) - 1),
        r#""p" must be a Mersenne prime, so "p == 2 ** s - 1" constraint should stand."#
    );
    let result = (x & P) + (x >> P_E);
    if result >= P {
        result - P
    } else {
        result
    }
}
