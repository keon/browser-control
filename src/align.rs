//! Power-of-two alignment over `usize`. `align` must be a non-zero power of two.

use crate::BitError;

/// True if `v` is a non-zero power of two.
#[inline] pub const fn is_power_of_two(v: usize) -> bool { v != 0 && (v & v.wrapping_sub(1)) == 0 }

#[inline]
const fn check(a: usize) -> Result<(), BitError> {
    if a == 0 { Err(BitError::AlignmentZero) }
    else if !is_power_of_two(a) { Err(BitError::AlignmentNotPowerOfTwo) }
    else { Ok(()) }
}

/// True if `v` is a multiple of `a`.
#[inline]
pub const fn is_aligned(v: usize, a: usize) -> Result<bool, BitError> {
    match check(a) { Ok(()) => Ok((v & (a - 1)) == 0), Err(e) => Err(e) }
}
/// Largest multiple of `a` that is `<= v`.
#[inline]
pub const fn down(v: usize, a: usize) -> Result<usize, BitError> {
    match check(a) { Ok(()) => Ok(v & !(a - 1)), Err(e) => Err(e) }
}
/// Smallest multiple of `a` that is `>= v`. [`BitError::Overflow`] if it would overflow.
#[inline]
pub const fn up(v: usize, a: usize) -> Result<usize, BitError> {
    match check(a) {
        Ok(()) => match v.checked_add(a - 1) {
            Some(s) => Ok(s & !(a - 1)),
            None => Err(BitError::Overflow),
        },
        Err(e) => Err(e),
    }
}
/// Bytes needed to reach the next multiple of `a`.
#[inline]
pub const fn padding(v: usize, a: usize) -> Result<usize, BitError> {
    match check(a) { Ok(()) => Ok(v.wrapping_neg() & (a - 1)), Err(e) => Err(e) }
}
