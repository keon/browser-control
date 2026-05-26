//! # bits
//!
//! Width-aware bit manipulation around a single [`Bits<T>`] newtype.
//!
//! ```
//! use bits::prelude::*;
//! let x = Bits::<u32>::new(0b1011_0000);
//! assert_eq!(x.isolate_lowest_set_bit(), Bits::new(0b0001_0000));
//! assert_eq!(x.set_bit(0)?,  Bits::new(0b1011_0001));
//! assert_eq!(x.extract(1..4)?, Bits::new(0b000));
//! # Ok::<(), bits::BitError>(())
//! ```
//!
//! ## Modules
//!
//! | module    | purpose                                             |
//! |-----------|-----------------------------------------------------|
//! | [`Bits`]  | The primary type; all bit operations are methods.   |
//! | [`Flags`] | Generic flag-set newtype.                           |
//! | [`align`] | Power-of-two alignment.                             |
//! | [`bytes`] | Read/write integers from `&[u8]` with endianness.   |
//! | [`mod@format`]| Allocation-free grouped-binary `Display`.           |
//! | [`explain`] (feature) | Educational metadata for bit hacks.     |

#![cfg_attr(not(feature = "std"), no_std)]
#![deny(unsafe_code)]
#![warn(missing_docs, rust_2018_idioms)]
#![allow(clippy::needless_doctest_main)]

#[cfg(all(feature = "alloc", not(feature = "std")))]
extern crate alloc;

mod accel;
mod bits;
pub mod align;
pub mod bytes;
pub mod format;
pub mod morton;

#[cfg(feature = "explain")]
pub mod explain;

pub use bits::Bits;

// ---------------------------------------------------------------------------
// BitError
// ---------------------------------------------------------------------------

/// Every fallible operation returns `Result<_, BitError>`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum BitError {
    /// Bit index `>= BITS`.
    IndexOutOfRange,
    /// `start > end`, or `end > BITS`.
    InvalidRange,
    /// Value does not fit in the target field width.
    FieldValueTooLarge,
    /// `align == 0`.
    AlignmentZero,
    /// `align` is not a power of two.
    AlignmentNotPowerOfTwo,
    /// Arithmetic overflow.
    Overflow,
    /// Operation undefined for zero (e.g. `next_power_of_two(0)`).
    UnderflowFromZero,
    /// Byte slice too short.
    InsufficientBytes,
}

impl core::fmt::Display for BitError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            BitError::IndexOutOfRange => "bit index out of range",
            BitError::InvalidRange => "invalid bit range",
            BitError::FieldValueTooLarge => "value does not fit in target bit field",
            BitError::AlignmentZero => "alignment must be nonzero",
            BitError::AlignmentNotPowerOfTwo => "alignment must be a power of two",
            BitError::Overflow => "arithmetic overflow",
            BitError::UnderflowFromZero => "operation undefined for zero",
            BitError::InsufficientBytes => "byte slice too short",
        })
    }
}

#[cfg(feature = "std")]
impl std::error::Error for BitError {}

// ---------------------------------------------------------------------------
// IntoBitRange
// ---------------------------------------------------------------------------

/// Accept either `Range<u32>` or `(u32, u32)` for bit-range arguments.
pub trait IntoBitRange {
    /// Lower to `(start, end)`; `start` inclusive, `end` exclusive.
    fn into_bit_range(self) -> (u32, u32);
}
impl IntoBitRange for core::ops::Range<u32> {
    #[inline]
    fn into_bit_range(self) -> (u32, u32) { (self.start, self.end) }
}
impl IntoBitRange for (u32, u32) {
    #[inline]
    fn into_bit_range(self) -> (u32, u32) { self }
}

// ---------------------------------------------------------------------------
// Flags<T>
// ---------------------------------------------------------------------------

/// Flag set wrapping a primitive unsigned integer.
///
/// For named, fixed sets of flags use the `bitflags` crate. `Flags<T>` is
/// for *ad hoc* flag manipulation where bit positions are data.
///
/// ```
/// use bits::Flags;
/// const READ: u32 = 1 << 0;
/// const WRITE: u32 = 1 << 1;
/// let mut f = Flags::<u32>::empty();
/// f.enable(READ | WRITE);
/// assert!(f.has(READ));
/// assert!(f.has_all(READ | WRITE));
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Flags<T>(pub T);

macro_rules! impl_flags {
    ($($ty:ty),*) => { $(
        impl Flags<$ty> {
            /// Empty set.
            #[inline] pub const fn empty() -> Self { Self(0) }
            /// All bits set.
            #[inline] pub const fn all() -> Self { Self(!(0 as $ty)) }
            /// Wrap raw bits.
            #[inline] pub const fn from_bits(b: $ty) -> Self { Self(b) }
            /// Underlying bits.
            #[inline] pub const fn bits(self) -> $ty { self.0 }
            /// Every bit of `flags` is set in `self`. (`has(0)` is `true`.)
            #[inline] pub const fn has(self, flags: $ty) -> bool { (self.0 & flags) == flags }
            /// Alias for [`has`](Self::has). Reads better at call sites with multiple bits.
            #[inline] pub const fn has_all(self, flags: $ty) -> bool { Self::has(self, flags) }
            /// Any bit of `flags` is set in `self`.
            #[inline] pub const fn has_any(self, flags: $ty) -> bool { (self.0 & flags) != 0 }
            /// Copy with `flags` set.
            #[inline] pub const fn with(self, flags: $ty) -> Self { Self(self.0 | flags) }
            /// Copy with `flags` cleared.
            #[inline] pub const fn without(self, flags: $ty) -> Self { Self(self.0 & !flags) }
            /// Copy with `flags` toggled.
            #[inline] pub const fn toggled(self, flags: $ty) -> Self { Self(self.0 ^ flags) }
            /// Set `flags` in place.
            #[inline] pub fn enable(&mut self, flags: $ty) { self.0 |= flags; }
            /// Clear `flags` in place.
            #[inline] pub fn disable(&mut self, flags: $ty) { self.0 &= !flags; }
            /// Toggle `flags` in place.
            #[inline] pub fn toggle(&mut self, flags: $ty) { self.0 ^= flags; }
        }
        impl From<$ty> for Flags<$ty> { #[inline] fn from(v: $ty) -> Self { Self(v) } }
        impl From<Flags<$ty>> for $ty { #[inline] fn from(f: Flags<$ty>) -> Self { f.0 } }
    )* };
}
impl_flags!(u8, u16, u32, u64, u128, usize);

// ---------------------------------------------------------------------------
// Prelude
// ---------------------------------------------------------------------------

/// `use bits::prelude::*;` to import the common types.
pub mod prelude {
    pub use crate::{BitError, Bits, Flags, IntoBitRange};
}
