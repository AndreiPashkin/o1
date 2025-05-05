//! General-purpose compile-time alternatives to existing non-const functions.

/// Calculates the ceiling of the division of two `f32` numbers at compile time.
pub const fn div_ceil_f32(a: f32, b: f32) -> i32 {
    // Convert to fixed-point with sufficient precision
    const SCALE: i32 = 1000000;
    let a_fixed = (a * SCALE as f32) as i32;
    let b_fixed = (b * SCALE as f32) as i32;

    // Integer division with ceiling
    let result = a_fixed / b_fixed;
    let remainder = a_fixed % b_fixed;

    if remainder > 0 {
        result + 1
    } else {
        result
    }
}
