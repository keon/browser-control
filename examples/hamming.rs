//! Hamming weight and distance — the count of differing bits between
//! two integers. Used by error-correcting codes, locality-sensitive
//! hashing, and approximate nearest-neighbor search.
//!
//! Run: `cargo run --example hamming`

use bitkit::prelude::*;

fn hamming_weight(x: u64) -> u32 {
    Bits::<u64>::new(x).count_ones()
}

fn hamming_distance(a: u64, b: u64) -> u32 {
    Bits::<u64>::new(a ^ b).count_ones()
}

fn nearest_in(corpus: &[u64], query: u64) -> (u64, u32) {
    corpus.iter()
        .map(|&c| (c, hamming_distance(c, query)))
        .min_by_key(|&(_, d)| d)
        .unwrap()
}

fn main() {
    let signatures: Vec<u64> = vec![
        0xCAFE_BABE_DEAD_BEEF,
        0x1234_5678_9ABC_DEF0,
        0xAAAA_AAAA_5555_5555,
        0xFFFF_FFFF_FFFF_FFFF,
        0x0000_0000_0000_0000,
        0x8000_0000_0000_0001,
    ];

    println!("Hamming weights:");
    for &s in &signatures {
        println!("  0x{s:016X}   weight = {:>2}", hamming_weight(s));
    }

    println!("\nDistance matrix:");
    print!("            ");
    for i in 0..signatures.len() { print!("  #{i}  "); }
    println!();
    for (i, &a) in signatures.iter().enumerate() {
        print!("  #{i}        ");
        for &b in &signatures {
            print!(" {:>4} ", hamming_distance(a, b));
        }
        println!();
    }

    let query = 0xCAFE_BABE_DEAD_C0FE; // close to entry 0
    let (nearest, d) = nearest_in(&signatures, query);
    println!("\nNearest-neighbor of 0x{query:016X}:");
    println!("  -> 0x{nearest:016X}  (Hamming distance {})", d);
}
