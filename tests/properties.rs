//! Property tests, parameterized over u8/u16/u32/u64/u128/usize.

use bits::prelude::*;
use proptest::prelude::*;

macro_rules! per_width {
    ($ty:ty, $modname:ident) => { mod $modname {
        use super::*;
        proptest! {
            #[test]
            fn set_clear_toggle_roundtrip(x: $ty, idx in 0u32..<$ty>::BITS) {
                let b = Bits::<$ty>::new(x);
                prop_assert!(b.set_bit(idx).unwrap().has_bit(idx).unwrap());
                prop_assert!(!b.clear_bit(idx).unwrap().has_bit(idx).unwrap());
                prop_assert_eq!(b.toggle_bit(idx).unwrap().toggle_bit(idx).unwrap(), b);
            }

            #[test]
            fn out_of_range_is_error(x: $ty, off in 0u32..1024) {
                let idx = <$ty>::BITS + off;
                let b = Bits::<$ty>::new(x);
                prop_assert_eq!(b.set_bit(idx),    Err(BitError::IndexOutOfRange));
                prop_assert_eq!(b.clear_bit(idx),  Err(BitError::IndexOutOfRange));
                prop_assert_eq!(b.toggle_bit(idx), Err(BitError::IndexOutOfRange));
                prop_assert_eq!(b.has_bit(idx),    Err(BitError::IndexOutOfRange));
            }

            #[test]
            fn isolate_lowest_set_has_one_or_zero(x: $ty) {
                let y = Bits::<$ty>::new(x).isolate_lowest_set_bit();
                prop_assert!(y.get() == 0 || y.count_ones() == 1);
            }

            #[test]
            fn clear_lowest_drops_one_bit(x: $ty) {
                if x != 0 {
                    let b = Bits::<$ty>::new(x);
                    prop_assert_eq!(b.clear_lowest_set_bit().count_ones(), b.count_ones() - 1);
                }
            }

            #[test]
            fn extract_insert_inverse(x: $ty, s in 0u32..<$ty>::BITS, w in 0u32..<$ty>::BITS, v: $ty) {
                let end = s.saturating_add(w).min(<$ty>::BITS);
                let width = end - s;
                let vm: $ty = if width == 0 { 0 }
                              else if width == <$ty>::BITS { !0 }
                              else { (1 as $ty).checked_shl(width).unwrap_or(0).wrapping_sub(1) };
                let value = v & vm;
                let b = Bits::<$ty>::new(x);
                let inserted = b.insert(s..end, value).unwrap();
                prop_assert_eq!(inserted.extract(s..end).unwrap().get(), value);
            }

            #[test]
            fn replace_truncates(x: $ty, s in 0u32..<$ty>::BITS, w in 0u32..<$ty>::BITS, v: $ty) {
                let end = s.saturating_add(w).min(<$ty>::BITS);
                let width = end - s;
                let vm: $ty = if width == 0 { 0 }
                              else if width == <$ty>::BITS { !0 }
                              else { (1 as $ty).checked_shl(width).unwrap_or(0).wrapping_sub(1) };
                let replaced = Bits::<$ty>::new(x).replace(s..end, v).unwrap();
                prop_assert_eq!(replaced.extract(s..end).unwrap().get(), v & vm);
            }

            #[test]
            fn low_mask_count(width in 0u32..=<$ty>::BITS) {
                prop_assert_eq!(Bits::<$ty>::low_mask(width).unwrap().count_ones(), width);
            }

            #[test]
            fn range_mask_lies_in_range(s in 0u32..=<$ty>::BITS, e in 0u32..=<$ty>::BITS) {
                let r = Bits::<$ty>::range_mask(s..e);
                if s > e {
                    prop_assert!(r.is_err());
                } else {
                    let m = r.unwrap();
                    prop_assert_eq!(m.count_ones(), e - s);
                    for i in 0..<$ty>::BITS {
                        prop_assert_eq!(m.has_bit(i).unwrap(), i >= s && i < e);
                    }
                }
            }

            #[test]
            fn set_bits_reconstruction(x: $ty) {
                let b = Bits::<$ty>::new(x);
                let mut r: $ty = 0;
                for i in b.set_bits() { r |= (1 as $ty) << i; }
                prop_assert_eq!(r, x);
            }

            #[test]
            fn submasks_count_equals_two_to_popcount(mask: $ty) {
                // Only verify for small popcounts to keep runtime sane.
                let b = Bits::<$ty>::new(mask);
                if b.count_ones() <= 8 {
                    prop_assert_eq!(b.submasks().count(), 1usize << b.count_ones());
                }
            }
        }
    } };
}

per_width!(u8,    p_u8);
per_width!(u16,   p_u16);
per_width!(u32,   p_u32);
per_width!(u64,   p_u64);
per_width!(u128,  p_u128);
per_width!(usize, p_usize);

// Non-width-specific properties.
proptest! {
    #[test]
    fn align_up_invariants(value: usize, log_a in 0u32..16) {
        let a = 1usize << log_a;
        match bits::align::up(value, a) {
            Ok(r) => {
                prop_assert!(r >= value);
                prop_assert_eq!(r % a, 0);
                prop_assert!(r - value < a);
            }
            Err(_) => prop_assert!(value.checked_add(a - 1).is_none()),
        }
    }

    #[test]
    fn align_down_invariants(value: usize, log_a in 0u32..16) {
        let a = 1usize << log_a;
        let r = bits::align::down(value, a).unwrap();
        prop_assert!(r <= value && r % a == 0 && value - r < a);
    }

    #[test]
    fn bytes_roundtrip_u32_be(v: u32) {
        let mut buf = [0u8; 4];
        bits::bytes::write_u32_be(&mut buf, v).unwrap();
        prop_assert_eq!(bits::bytes::read_u32_be(&buf).unwrap(), v);
    }

    #[test]
    fn bytes_roundtrip_u128_le(v: u128) {
        let mut buf = [0u8; 16];
        bits::bytes::write_u128_le(&mut buf, v).unwrap();
        prop_assert_eq!(bits::bytes::read_u128_le(&buf).unwrap(), v);
    }
}

// ---- gather / scatter properties -------------------------------------

proptest! {
    /// `gather(x, mask).scatter(mask) == x & mask`
    #[test]
    fn gather_scatter_round_trip_u32(x: u32, mask: u32) {
        let bx = Bits::<u32>::new(x);
        let bm = Bits::<u32>::new(mask);
        prop_assert_eq!(bx.gather(bm).scatter(bm).get(), x & mask);
    }

    /// `gather(x, mask).count_ones() <= popcount(mask)` and equals
    /// `popcount(x & mask)`.
    #[test]
    fn gather_count_matches_masked_popcount_u32(x: u32, mask: u32) {
        let g = Bits::<u32>::new(x).gather(Bits::new(mask));
        prop_assert_eq!(g.count_ones(), (x & mask).count_ones());
        // Result occupies only the low popcount(mask) bits.
        prop_assert_eq!(g.get() >> mask.count_ones(), 0);
    }

    /// `scatter(low, mask).gather(mask) == low & low_mask(popcount(mask))`
    #[test]
    fn scatter_gather_low_bits_u32(low: u32, mask: u32) {
        let p = mask.count_ones();
        let lm = if p == 32 { u32::MAX } else if p == 0 { 0 } else { (1u32 << p) - 1 };
        let r = Bits::<u32>::new(low).scatter(Bits::new(mask)).gather(Bits::new(mask));
        prop_assert_eq!(r.get(), low & lm);
    }

    /// Scatter populates only positions in mask.
    #[test]
    fn scatter_subset_of_mask_u32(low: u32, mask: u32) {
        let s = Bits::<u32>::new(low).scatter(Bits::new(mask)).get();
        prop_assert_eq!(s & !mask, 0);
    }

    // u64 variants

    #[test]
    fn gather_scatter_round_trip_u64(x: u64, mask: u64) {
        let bx = Bits::<u64>::new(x);
        let bm = Bits::<u64>::new(mask);
        prop_assert_eq!(bx.gather(bm).scatter(bm).get(), x & mask);
    }

    #[test]
    fn scatter_subset_of_mask_u64(low: u64, mask: u64) {
        let s = Bits::<u64>::new(low).scatter(Bits::new(mask)).get();
        prop_assert_eq!(s & !mask, 0);
    }
}

// ---- new primitives ---------------------------------------------------

proptest! {
    #[test]
    fn reverse_bits_involution_u32(x: u32) {
        let b = Bits::<u32>::new(x);
        prop_assert_eq!(b.reverse_bits().reverse_bits(), b);
    }

    #[test]
    fn blsmsk_clears_above_lowest_set_u32(x in 1u32..) {
        let b = Bits::<u32>::new(x).mask_through_lowest_set_bit();
        // Bits above the lowest set bit must be zero.
        let lz = x.trailing_zeros();
        prop_assert!(b.get() >> (lz + 1) == 0);
        // The bit at and below `lz` are set.
        prop_assert_eq!(b.get(), (1u64 << (lz + 1)).saturating_sub(1) as u32);
    }

    #[test]
    fn bzhi_keeps_low_n_u32(x: u32, n in 0u32..=32) {
        let r = Bits::<u32>::new(x).zero_high_bits(n).unwrap().get();
        let mask = if n == 32 { u32::MAX } else { (1u32 << n) - 1 };
        prop_assert_eq!(r, x & mask);
    }

    #[test]
    fn morton_2d_roundtrip(x: u16, y: u16) {
        let z = bits::morton::encode_2d(x, y);
        prop_assert_eq!(bits::morton::decode_2d(z), (x, y));
    }

    #[test]
    fn morton_2d_u64_roundtrip(x: u32, y: u32) {
        let z = bits::morton::encode_2d_u64(x, y);
        prop_assert_eq!(bits::morton::decode_2d_u64(z), (x, y));
    }

    #[test]
    fn morton_3d_roundtrip(x in 0u16..1024, y in 0u16..1024, z in 0u16..1024) {
        let m = bits::morton::encode_3d(x, y, z);
        prop_assert_eq!(bits::morton::decode_3d(m), (x, y, z));
    }
}
