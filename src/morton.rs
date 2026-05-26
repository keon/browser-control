//! Morton (Z-order) curve encode / decode.
//!
//! Interleaves the bits of multiple coordinate streams into a single
//! integer key. Spatial-locality trick used by quadtrees / octrees,
//! GPU texture caches, range queries, and Hilbert-curve variants.
//!
//! Implementations use [`Bits::scatter`](crate::Bits::scatter) /
//! [`gather`](crate::Bits::gather) underneath, which lower to one
//! `PDEP`/`PEXT` instruction on x86_64 BMI2 (Intel ≥ Haswell, AMD ≥
//! Zen 3). On other targets they fall back to the portable SWAR path —
//! still faster than the bit-by-bit loop used by `morton-encoding`.
//!
//! ## Example
//!
//! ```
//! use bitkit::morton;
//! assert_eq!(morton::encode_2d(1234, 5678), 37247404);
//! assert_eq!(morton::decode_2d(37247404), (1234, 5678));
//! ```

use crate::Bits;

// 2D lane masks: every other bit.
const LANE_2D_X: u32 = 0x5555_5555;
const LANE_2D_Y: u32 = 0xAAAA_AAAA;

// 2D wide lane masks for u64.
const LANE_2D_X_U64: u64 = 0x5555_5555_5555_5555;
const LANE_2D_Y_U64: u64 = 0xAAAA_AAAA_AAAA_AAAA;

// 3D lane masks: every third bit, offset 0 / 1 / 2.
// Bits set at positions {0,3,6,9,...} etc. Pattern repeats every 3 bits.
const LANE_3D_X: u32 = 0x0924_9249; // 001 001 001 ...
const LANE_3D_Y: u32 = 0x1249_2492; // 010 010 010 ...
const LANE_3D_Z: u32 = 0x2492_4924; // 100 100 100 ...

/// Encode 2D coordinates `(x, y)` into a 32-bit Morton key.
///
/// Each coordinate occupies 16 bits of the result.
#[inline]
pub fn encode_2d(x: u16, y: u16) -> u32 {
    (Bits::<u32>::new(x as u32).scatter(Bits::new(LANE_2D_X))
     | Bits::<u32>::new(y as u32).scatter(Bits::new(LANE_2D_Y))).get()
}

/// Decode a 32-bit Morton key back into `(x, y)`.
#[inline]
pub fn decode_2d(z: u32) -> (u16, u16) {
    let zb = Bits::<u32>::new(z);
    (
        zb.gather(Bits::new(LANE_2D_X)).get() as u16,
        zb.gather(Bits::new(LANE_2D_Y)).get() as u16,
    )
}

/// Encode 2D 32-bit coordinates `(x, y)` into a 64-bit Morton key.
#[inline]
pub fn encode_2d_u64(x: u32, y: u32) -> u64 {
    (Bits::<u64>::new(x as u64).scatter(Bits::new(LANE_2D_X_U64))
     | Bits::<u64>::new(y as u64).scatter(Bits::new(LANE_2D_Y_U64))).get()
}

/// Decode a 64-bit Morton key into a `(x, y)` pair of 32-bit coordinates.
#[inline]
pub fn decode_2d_u64(z: u64) -> (u32, u32) {
    let zb = Bits::<u64>::new(z);
    (
        zb.gather(Bits::new(LANE_2D_X_U64)).get() as u32,
        zb.gather(Bits::new(LANE_2D_Y_U64)).get() as u32,
    )
}

/// Encode 3D coordinates `(x, y, z)` into a 32-bit Morton key.
///
/// Each coordinate must fit in **10 bits** (`< 1024`); higher bits are
/// silently truncated.
#[inline]
pub fn encode_3d(x: u16, y: u16, z: u16) -> u32 {
    let m10 = Bits::<u32>::new(0x3FF); // low 10-bit mask
    let xb = Bits::<u32>::new(x as u32) & m10;
    let yb = Bits::<u32>::new(y as u32) & m10;
    let zb = Bits::<u32>::new(z as u32) & m10;
    (xb.scatter(Bits::new(LANE_3D_X))
     | yb.scatter(Bits::new(LANE_3D_Y))
     | zb.scatter(Bits::new(LANE_3D_Z))).get()
}

/// Decode a 32-bit 3D Morton key into `(x, y, z)`, each 10 bits.
#[inline]
pub fn decode_3d(w: u32) -> (u16, u16, u16) {
    let wb = Bits::<u32>::new(w);
    (
        wb.gather(Bits::new(LANE_3D_X)).get() as u16,
        wb.gather(Bits::new(LANE_3D_Y)).get() as u16,
        wb.gather(Bits::new(LANE_3D_Z)).get() as u16,
    )
}
