use std::num::one;

/// Compute integer division and modulus, rounding down.
/// Contrast with div_rem.
pub fn div_mod<T: Signed>(x: T, y: T) -> (T, T) {
    let quot = x / y;
    let rem = x % y;
    if rem.is_negative() {
        (quot - one(), rem + y)
    } else {
        (quot, rem)
    }
}
