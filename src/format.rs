//! Allocation-free grouped-binary [`Display`](core::fmt::Display) wrapper.
//!
//! For ungrouped binary use the standard `{:b}` — [`Bits`]
//! implements [`Binary`](core::fmt::Binary).
//!
//! ```
//! use bitkit::{format::grouped_binary, Bits};
//! assert_eq!(format!("{}", grouped_binary(Bits::<u8>::new(0xA5), 4)), "1010_0101");
//! ```

use crate::Bits;
use core::fmt;

/// `Display` wrapper that renders [`Bits`] as `group`-bit groups.
pub struct GroupedBinary<T> { value: Bits<T>, group: u32 }

macro_rules! impl_disp { ($ty:ty) => {
    impl fmt::Display for GroupedBinary<$ty> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let bits = <$ty>::BITS;
            let mut i = bits;
            while i > 0 {
                i -= 1;
                if i != bits - 1 && self.group != 0 && (i + 1) % self.group == 0 { f.write_str("_")?; }
                f.write_str(if (self.value.0 >> i) & 1 == 0 { "0" } else { "1" })?;
            }
            Ok(())
        }
    }
}; }
impl_disp!(u8); impl_disp!(u16); impl_disp!(u32); impl_disp!(u64); impl_disp!(u128); impl_disp!(usize);

/// Wrap `x` in a `Display` that uses `group`-bit groups (`0` = no grouping).
#[inline] pub const fn grouped_binary<T>(x: Bits<T>, group: u32) -> GroupedBinary<T> { GroupedBinary { value: x, group } }
