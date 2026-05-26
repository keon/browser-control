//! API surface + deterministic examples.

use bitkit::prelude::*;
use bitkit::{align, bytes, format::grouped_binary};

#[test]
fn width_constants() {
    assert_eq!(Bits::<u8>::BITS, 8);
    assert_eq!(Bits::<u32>::BITS, 32);
    assert_eq!(Bits::<u128>::BITS, 128);
    assert_eq!(Bits::<usize>::BITS, usize::BITS);
}

#[test]
fn single_bit_ops_return_result() {
    let x = Bits::<u32>::new(0);
    assert!(matches!(x.set_bit(40), Err(BitError::IndexOutOfRange)));
    let y = x.set_bit(0).unwrap();
    assert!(y.has_bit(0).unwrap());
    assert_eq!(y.toggle_bit(0).unwrap(), x);
}

#[test]
fn wrapping_variants() {
    let x = Bits::<u8>::new(0);
    assert_eq!(x.set_bit_wrapping(0).get(), 1);
    assert_eq!(x.set_bit_wrapping(8).get(), 1);
    assert_eq!(x.set_bit_wrapping(9).get(), 0b10);
}

#[test]
fn bit_hacks() {
    let x = Bits::<u32>::new(0b1011_0000);
    assert_eq!(x.isolate_lowest_set_bit().get(), 0b0001_0000);
    assert_eq!(x.clear_lowest_set_bit().get(),   0b1010_0000);
    assert_eq!(Bits::<u32>::new(0b1011).isolate_lowest_zero_bit().get(), 0b0100);
    assert_eq!(Bits::<u32>::new(0b0111).set_lowest_zero_bit().get(),     0b1111);
}

#[test]
fn power_of_two_no_silent_zero() {
    assert!(Bits::<u32>::new(1).is_power_of_two());
    assert!(!Bits::<u32>::new(0).is_power_of_two());
    assert_eq!(Bits::<u32>::new(0).next_power_of_two(),     Err(BitError::UnderflowFromZero));
    assert_eq!(Bits::<u32>::new(0).previous_power_of_two(), Err(BitError::UnderflowFromZero));
    assert_eq!(Bits::<u32>::new(5).next_power_of_two().unwrap().get(),     8);
    assert_eq!(Bits::<u32>::new(5).previous_power_of_two().unwrap().get(), 4);
    assert_eq!(Bits::<u32>::new((1u32 << 31) + 1).next_power_of_two(), Err(BitError::Overflow));
}

#[test]
fn masks_via_method() {
    assert_eq!(Bits::<u8>::low_mask(3).unwrap().get(),     0b0000_0111);
    assert_eq!(Bits::<u8>::low_mask(8).unwrap().get(),     0xFF);
    assert!(Bits::<u8>::low_mask(9).is_err());
    assert_eq!(Bits::<u8>::high_mask(2).unwrap().get(),    0b1100_0000);
    assert_eq!(Bits::<u8>::range_mask(2..5).unwrap().get(), 0b0001_1100);
    assert_eq!(Bits::<u8>::range_mask((2, 5)).unwrap().get(), 0b0001_1100);
}

#[test]
fn fields() {
    let x = Bits::<u8>::new(0b1101_0110);
    assert_eq!(x.extract(1..4).unwrap().get(), 0b011);
    assert_eq!(x.insert(1..4, 0b101).unwrap().get(), 0b1101_1010);
    assert_eq!(x.replace(1..4, 0b1101).unwrap().get(), 0b1101_1010);
    assert_eq!(x.insert(1..4, 0b1000), Err(BitError::FieldValueTooLarge));
    assert_eq!(x.insert((5u32, 3u32), 0), Err(BitError::InvalidRange));
}

#[test]
fn alignment() {
    assert!(align::is_power_of_two(64));
    assert_eq!(align::is_aligned(16, 8), Ok(true));
    assert_eq!(align::is_aligned(0, 6),  Err(BitError::AlignmentNotPowerOfTwo));
    assert_eq!(align::down(13, 8),    Ok(8));
    assert_eq!(align::up(13, 8),      Ok(16));
    assert_eq!(align::up(usize::MAX, 8), Err(BitError::Overflow));
    assert_eq!(align::padding(13, 8), Ok(3));
}

#[test]
fn flags_generic() {
    const A: u32 = 0b001;
    const B: u32 = 0b010;
    let mut f = Flags::<u32>::empty();
    f.enable(A | B);
    assert!(f.has(A));
    assert!(f.has_all(A | B));
    assert!(f.has_any(A | 0xFF00));
    f.disable(A);
    assert!(!f.has(A));
    assert_eq!(Flags::<u32>::all().bits(), u32::MAX);
    let raw: u32 = Flags::<u32>::from_bits(0xAB).into();
    assert_eq!(raw, 0xAB);
}

#[test]
fn iterators() {
    let x = Bits::<u8>::new(0b1011);
    assert_eq!(x.set_bits().collect::<Vec<_>>(), vec![0, 1, 3]);
    assert_eq!(x.set_bit_values().collect::<Vec<_>>(), vec![1, 2, 8]);
    assert_eq!(Bits::<u8>::new(!0b1011u8).zero_bits().collect::<Vec<_>>(), vec![0, 1, 3]);
    let mut subs: Vec<u8> = Bits::<u8>::new(0b101).submasks().map(|b| b.get()).collect();
    subs.sort();
    assert_eq!(subs, vec![0, 1, 4, 5]);
    let proper: Vec<u8> = Bits::<u8>::new(0b101).proper_submasks().map(|b| b.get()).collect();
    assert!(!proper.contains(&0b101) && proper.contains(&0));
}

#[test]
fn bytes_mod() {
    let buf = [0x12, 0x34, 0x56, 0x78];
    assert_eq!(bytes::read_u16_be(&buf).unwrap(), 0x1234);
    assert_eq!(bytes::read_u16_le(&buf).unwrap(), 0x3412);
    assert_eq!(bytes::read_u32_be(&buf).unwrap(), 0x12345678);
    assert_eq!(bytes::read_u8(&buf).unwrap(), 0x12);
    assert_eq!(bytes::read_u32_be(&buf[..3]), Err(BitError::InsufficientBytes));

    let mut out = [0u8; 8];
    bytes::write_u64_be(&mut out, 0x0102_0304_0506_0708).unwrap();
    assert_eq!(out, [1,2,3,4,5,6,7,8]);
    let mut tiny = [0u8; 2];
    assert_eq!(bytes::write_u32_be(&mut tiny, 0), Err(BitError::InsufficientBytes));
}

#[test]
fn format_and_operators() {
    let x = Bits::<u32>::new(0b1011_0000);
    assert_eq!(format!("{}", grouped_binary(x, 4)),
        "0000_0000_0000_0000_0000_0000_1011_0000");
    assert_eq!(format!("{:08b}", Bits::<u8>::new(0b1011)), "00001011");
    assert_eq!(format!("{:x}", Bits::<u32>::new(0xCAFE)), "cafe");
    assert_eq!(format!("{:?}", Bits::<u32>::new(42)), "Bits(42)");

    let a = Bits::<u32>::new(0b1100);
    let b = Bits::<u32>::new(0b1010);
    assert_eq!((a & b).get(), 0b1000);
    assert_eq!((a | b).get(), 0b1110);
    assert_eq!((a ^ b).get(), 0b0110);
    assert_eq!((!a).get(), !0b1100u32);
}

#[test]
fn error_display() {
    assert!(format!("{}", BitError::IndexOutOfRange).contains("index"));
    assert!(format!("{}", BitError::UnderflowFromZero).contains("zero"));
}

#[cfg(feature = "explain")]
#[test]
fn explanations() {
    assert_eq!(bitkit::explain::isolate_lowest_set_bit().formula, "x & -x");
}

#[test]
fn const_friendly_indices_forms() {
    // These compile in const context — verify by using them in a const block.
    const M: bitkit::Bits<u32> = match bitkit::Bits::<u32>::range_mask_indices(2, 5) {
        Ok(m) => m, Err(_) => panic!(),
    };
    assert_eq!(M.get(), 0b0001_1100);

    const X: bitkit::Bits<u8> = bitkit::Bits::<u8>::new(0b1101_0110);
    const F: bitkit::Bits<u8> = match X.extract_indices(1, 4) {
        Ok(v) => v, Err(_) => panic!(),
    };
    assert_eq!(F.get(), 0b011);
}

#[test]
fn const_get() {
    const VAL: u32 = bitkit::Bits::<u32>::new(42).get();
    assert_eq!(VAL, 42);
}

#[test]
fn gather_scatter_u32_round_trip() {
    let v = Bits::<u32>::new(0xCAFE_BABE);
    let mask = Bits::<u32>::new(0xF0F0_F0F0);

    let g = v.gather(mask);
    // gather packs the bits of `v & mask` into the low bits.
    assert_eq!(g.count_ones(), (v & mask).count_ones());
    // gather then scatter on the same mask returns the masked-relevant bits of v.
    let s = g.scatter(mask);
    assert_eq!(s.get(), v.get() & mask.get());
}

#[test]
fn gather_scatter_u64_round_trip() {
    let v = Bits::<u64>::new(0xCAFE_BABE_DEAD_BEEF);
    let mask = Bits::<u64>::new(0x5555_AAAA_5555_AAAA);
    let g = v.gather(mask);
    assert_eq!(g.count_ones(), (v & mask).count_ones());
    assert_eq!(g.scatter(mask).get(), v.get() & mask.get());
}

#[test]
fn morton_via_scatter() {
    // 2D Morton encode: interleave bits of x (even lanes) and y (odd lanes).
    let x = Bits::<u32>::new(0b0000_0000_0000_0000_0000_0000_0000_1011);
    let y = Bits::<u32>::new(0b0000_0000_0000_0000_0000_0000_0000_1100);

    let x_mask = Bits::<u32>::new(0x5555_5555); // 0101...
    let y_mask = Bits::<u32>::new(0xAAAA_AAAA); // 1010...

    let morton = x.scatter(x_mask) | y.scatter(y_mask);

    // Decode the other direction.
    let x_dec = morton.gather(x_mask);
    let y_dec = morton.gather(y_mask);
    assert_eq!(x_dec, x);
    assert_eq!(y_dec, y);
}

#[test]
fn reverse_bits_and_blsmsk_and_bzhi() {
    assert_eq!(Bits::<u8>::new(0b0000_0001).reverse_bits().get(), 0b1000_0000);
    assert_eq!(Bits::<u32>::new(0xCAFE_BABE).reverse_bits().get(), 0xCAFE_BABEu32.reverse_bits());

    // BLSMSK: mask through lowest set bit. 0b1100 -> 0b0111.
    assert_eq!(Bits::<u32>::new(0b1100).mask_through_lowest_set_bit().get(), 0b0111);
    // BLSMSK of 0 == !0 (because 0 ^ (0-1)) — documented edge case.
    assert_eq!(Bits::<u32>::new(0).mask_through_lowest_set_bit().get(), u32::MAX);

    // BZHI: keep low n bits.
    assert_eq!(Bits::<u32>::new(0xFFFF_FFFF).zero_high_bits(8).unwrap().get(), 0xFF);
    assert_eq!(Bits::<u32>::new(0xFFFF_FFFF).zero_high_bits(0).unwrap().get(), 0);
    assert_eq!(Bits::<u32>::new(0xFFFF_FFFF).zero_high_bits(32).unwrap().get(), 0xFFFF_FFFF);
    assert_eq!(Bits::<u32>::new(0).zero_high_bits(33), Err(BitError::IndexOutOfRange));
}

#[test]
fn morton_2d_and_3d() {
    use bitkit::morton;
    // 2D round-trip
    for (x, y) in [(0u16, 0u16), (1, 0), (0, 1), (1234, 5678), (u16::MAX, u16::MAX)] {
        let z = morton::encode_2d(x, y);
        assert_eq!(morton::decode_2d(z), (x, y));
    }
    // 2D matches the bit-by-bit definition
    let (x, y) = (0xABCDu16, 0x1234u16);
    let z = morton::encode_2d(x, y);
    let manual: u32 = (0..16u32).fold(0, |acc, i|
        acc | (((x as u32 >> i) & 1) << (2 * i))
            | (((y as u32 >> i) & 1) << (2 * i + 1)));
    assert_eq!(z, manual);

    // 2D u64
    let zw = morton::encode_2d_u64(0xDEAD_BEEFu32, 0xCAFE_BABEu32);
    assert_eq!(morton::decode_2d_u64(zw), (0xDEAD_BEEF, 0xCAFE_BABE));

    // 3D round-trip — coordinates must fit in 10 bits.
    for (x, y, z) in [(0u16, 0u16, 0u16), (1023, 0, 0), (0, 1023, 0), (0, 0, 1023), (511, 512, 513)] {
        let m = morton::encode_3d(x, y, z);
        assert_eq!(morton::decode_3d(m), (x, y, z));
    }
}

#[test]
fn iter_for_each_semantics() {
    let x = Bits::<u8>::new(0b1011);
    let mut idxs = vec![];
    x.set_bits().for_each(|i| idxs.push(i));
    assert_eq!(idxs, vec![0, 1, 3]);

    let mut vals = vec![];
    x.set_bit_values().for_each(|v| vals.push(v));
    assert_eq!(vals, vec![1u8, 2, 8]);

    let mut zeros = vec![];
    Bits::<u8>::new(!0b1011u8).zero_bits().for_each(|i| zeros.push(i));
    assert_eq!(zeros, vec![0, 1, 3]);

    let mut subs = vec![];
    Bits::<u8>::new(0b101).submasks().for_each(|s| subs.push(s.get()));
    subs.sort();
    assert_eq!(subs, vec![0, 1, 4, 5]);

    let mut prop = vec![];
    Bits::<u8>::new(0b101).proper_submasks().for_each(|s| prop.push(s.get()));
    assert!(!prop.contains(&0b101));
    assert!(prop.contains(&0));
}
