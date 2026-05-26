//! Color packing — RGBA8888 ↔ u32 and RGB565 ↔ u16.
//!
//! Real-world bit-field work: most GPU formats pack subpixel channels
//! into a single integer. RGB565 splits a u16 into 5+6+5 bits;
//! RGBA8888 packs four bytes into a u32.
//!
//! Run: `cargo run --example rgb_pack`

use bitkit::prelude::*;

fn pack_rgba8888(r: u8, g: u8, b: u8, a: u8) -> u32 {
    Bits::<u32>::new(0)
        .insert(24..32, r as u32).unwrap()
        .insert(16..24, g as u32).unwrap()
        .insert( 8..16, b as u32).unwrap()
        .insert( 0.. 8, a as u32).unwrap()
        .get()
}

fn unpack_rgba8888(px: u32) -> (u8, u8, u8, u8) {
    let p = Bits::<u32>::new(px);
    (
        p.extract(24..32).unwrap().get() as u8,
        p.extract(16..24).unwrap().get() as u8,
        p.extract( 8..16).unwrap().get() as u8,
        p.extract( 0.. 8).unwrap().get() as u8,
    )
}

fn pack_rgb565(r: u8, g: u8, b: u8) -> u16 {
    // 5+6+5 bits — quantize 8-bit channels.
    let r5 = (r >> 3) as u16;
    let g6 = (g >> 2) as u16;
    let b5 = (b >> 3) as u16;
    Bits::<u16>::new(0)
        .insert(11..16, r5).unwrap()
        .insert( 5..11, g6).unwrap()
        .insert( 0.. 5, b5).unwrap()
        .get()
}

fn unpack_rgb565(px: u16) -> (u8, u8, u8) {
    let p = Bits::<u16>::new(px);
    let r5 = p.extract(11..16).unwrap().get() as u8;
    let g6 = p.extract( 5..11).unwrap().get() as u8;
    let b5 = p.extract( 0.. 5).unwrap().get() as u8;
    // Expand back to 8 bits per channel (replicate high bits).
    let r = (r5 << 3) | (r5 >> 2);
    let g = (g6 << 2) | (g6 >> 4);
    let b = (b5 << 3) | (b5 >> 2);
    (r, g, b)
}

fn main() {
    let orange = (0xFFu8, 0xA5u8, 0x00u8, 0xFFu8);
    let packed = pack_rgba8888(orange.0, orange.1, orange.2, orange.3);
    println!("RGBA8888 pack/unpack:");
    println!("  ({:>3}, {:>3}, {:>3}, {:>3})  ->  0x{:08X}",
        orange.0, orange.1, orange.2, orange.3, packed);
    assert_eq!(unpack_rgba8888(packed), orange);

    let teal = (0x00u8, 0x80u8, 0x80u8);
    let p565 = pack_rgb565(teal.0, teal.1, teal.2);
    let round = unpack_rgb565(p565);
    println!("\nRGB565 pack/unpack (5+6+5 quantization is lossy):");
    println!("  ({:>3}, {:>3}, {:>3})  ->  0x{:04X}  ->  ({:>3}, {:>3}, {:>3})",
        teal.0, teal.1, teal.2, p565, round.0, round.1, round.2);

    // Demonstrate the quantization range.
    let total: u64 = (0..=255u32).map(|c| {
        let p = pack_rgb565(c as u8, c as u8, c as u8);
        let (r, _, _) = unpack_rgb565(p);
        (c as i32 - r as i32).unsigned_abs() as u64
    }).sum();
    println!("\nAvg RGB565 round-trip error per channel (256 greyscale samples): {} levels.",
        total / 256);
}
