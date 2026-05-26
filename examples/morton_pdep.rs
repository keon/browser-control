//! Morton (Z-order) codes via the high-level `bits::morton` module.
//!
//! Underneath, each call uses `Bits::scatter` / `gather` which compile to
//! a single PDEP/PEXT instruction on x86_64 BMI2 (Intel ≥ Haswell, AMD ≥
//! Zen 3) and to a portable SWAR fallback elsewhere.
//!
//! Run:  `cargo run --example morton_pdep`

use bits::morton;

fn main() {
    println!("4x4 quadrant of 2D Morton codes:");
    println!("        x=0    x=1    x=2    x=3");
    for y in 0..4 {
        print!("y={y}:");
        for x in 0..4 {
            print!(" {:>6}", morton::encode_2d(x, y));
        }
        println!();
    }

    let (x, y) = (1234u16, 5678u16);
    let z = morton::encode_2d(x, y);
    let (x2, y2) = morton::decode_2d(z);
    println!("\nencode_2d({x}, {y})    = {z}");
    println!("decode_2d({z}) = ({x2}, {y2})");
    assert_eq!((x, y), (x2, y2));

    // 3D for octrees: each coord in 0..1024
    let (a, b, c) = (511u16, 512u16, 513u16);
    let m = morton::encode_3d(a, b, c);
    let (a2, b2, c2) = morton::decode_3d(m);
    println!("\nencode_3d({a}, {b}, {c}) = {m}");
    println!("decode_3d({m})    = ({a2}, {b2}, {c2})");
    assert_eq!((a, b, c), (a2, b2, c2));
}
