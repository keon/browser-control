//! Morton / Z-order codes: interleave the bits of two 16-bit coordinates
//! into a single 32-bit key. Spatial-locality trick used by quadtrees,
//! GPU texture caches, and the Bit Twiddling Hacks page.
//!
//! Run: `cargo run --example morton_code`

use bitkit::{format::grouped_binary, prelude::*};

fn morton_encode(x: u16, y: u16) -> u32 {
    let mut out = Bits::<u32>::new(0);
    let bx = Bits::<u16>::new(x);
    let by = Bits::<u16>::new(y);
    for i in 0..16 {
        if bx.has_bit(i).unwrap() {
            out = out.set_bit(2 * i).unwrap();
        }
        if by.has_bit(i).unwrap() {
            out = out.set_bit(2 * i + 1).unwrap();
        }
    }
    out.get()
}

fn morton_decode(z: u32) -> (u16, u16) {
    let mut x = Bits::<u16>::new(0);
    let mut y = Bits::<u16>::new(0);
    let bz = Bits::<u32>::new(z);
    for i in 0..16 {
        if bz.has_bit(2 * i).unwrap() {
            x = x.set_bit(i).unwrap();
        }
        if bz.has_bit(2 * i + 1).unwrap() {
            y = y.set_bit(i).unwrap();
        }
    }
    (x.get(), y.get())
}

fn main() {
    println!("4x4 quadrant of Morton codes:");
    println!("        x=0    x=1    x=2    x=3");
    for y in 0..4 {
        print!("y={y}:");
        for x in 0..4 {
            print!(" {:>6}", morton_encode(x, y));
        }
        println!();
    }

    println!();
    let (x, y) = (1234u16, 5678u16);
    let z = morton_encode(x, y);
    let (x2, y2) = morton_decode(z);
    println!("encode({x}, {y}) = {z}");
    println!("              = {}", grouped_binary(Bits::<u32>::new(z), 4));
    println!("decode({z})    = ({x2}, {y2})");
    assert_eq!((x, y), (x2, y2));
}
