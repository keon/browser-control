//! Read/write integers from `&[u8]` with explicit endianness. Never panics on short input.

use crate::BitError;

macro_rules! io { ($ty:ty, $n:literal, $rb:ident, $rl:ident, $wb:ident, $wl:ident) => {
    /// Read big-endian.
    #[inline]
    pub const fn $rb(buf: &[u8]) -> Result<$ty, BitError> {
        if buf.len() < $n { return Err(BitError::InsufficientBytes); }
        let mut b = [0u8; $n]; let mut i = 0;
        while i < $n { b[i] = buf[i]; i += 1; }
        Ok(<$ty>::from_be_bytes(b))
    }
    /// Read little-endian.
    #[inline]
    pub const fn $rl(buf: &[u8]) -> Result<$ty, BitError> {
        if buf.len() < $n { return Err(BitError::InsufficientBytes); }
        let mut b = [0u8; $n]; let mut i = 0;
        while i < $n { b[i] = buf[i]; i += 1; }
        Ok(<$ty>::from_le_bytes(b))
    }
    /// Write big-endian.
    #[inline]
    pub fn $wb(out: &mut [u8], v: $ty) -> Result<(), BitError> {
        if out.len() < $n { return Err(BitError::InsufficientBytes); }
        out[..$n].copy_from_slice(&v.to_be_bytes()); Ok(())
    }
    /// Write little-endian.
    #[inline]
    pub fn $wl(out: &mut [u8], v: $ty) -> Result<(), BitError> {
        if out.len() < $n { return Err(BitError::InsufficientBytes); }
        out[..$n].copy_from_slice(&v.to_le_bytes()); Ok(())
    }
}; }
io!(u16,  2, read_u16_be, read_u16_le, write_u16_be, write_u16_le);
io!(u32,  4, read_u32_be, read_u32_le, write_u32_be, write_u32_le);
io!(u64,  8, read_u64_be, read_u64_le, write_u64_be, write_u64_le);
io!(u128, 16, read_u128_be, read_u128_le, write_u128_be, write_u128_le);

/// Read a single byte.
#[inline]
pub const fn read_u8(buf: &[u8]) -> Result<u8, BitError> {
    if buf.is_empty() { Err(BitError::InsufficientBytes) } else { Ok(buf[0]) }
}
/// Write a single byte.
#[inline]
pub fn write_u8(out: &mut [u8], v: u8) -> Result<(), BitError> {
    if out.is_empty() { Err(BitError::InsufficientBytes) } else { out[0] = v; Ok(()) }
}
