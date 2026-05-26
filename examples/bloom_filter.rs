//! A tiny Bloom filter built on a `Vec<u64>` indexed via bit operations.
//!
//! No false negatives, tunable false-positive rate. Inserting and
//! testing both reduce to "hash → bit index → set/test that bit."
//!
//! Run: `cargo run --example bloom_filter`

#![allow(clippy::unusual_byte_groupings)]

use bitkit::prelude::*;

struct Bloom {
    bits: Vec<u64>, // each u64 holds 64 slots
    bit_count: usize,
    hashes: u32,
}

impl Bloom {
    fn new(bit_count: usize, hashes: u32) -> Self {
        let words = bit_count.div_ceil(64);
        Self { bits: vec![0u64; words], bit_count, hashes }
    }

    fn slot_for(&self, value: u64, k: u32) -> usize {
        // Double-hashing: h1 + k * h2  (FNV-ish constants).
        let h1 = value.wrapping_mul(0x100000001b3);
        let h2 = value.wrapping_mul(0x9E37_79B97F4A_7C15).rotate_left(13);
        ((h1.wrapping_add(h2.wrapping_mul(k as u64))) as usize) % self.bit_count
    }

    fn insert(&mut self, value: u64) {
        for k in 0..self.hashes {
            let i = self.slot_for(value, k);
            let (word, bit) = (i / 64, (i % 64) as u32);
            self.bits[word] = Bits::<u64>::new(self.bits[word]).set_bit(bit).unwrap().get();
        }
    }

    fn contains(&self, value: u64) -> bool {
        for k in 0..self.hashes {
            let i = self.slot_for(value, k);
            let (word, bit) = (i / 64, (i % 64) as u32);
            if !Bits::<u64>::new(self.bits[word]).has_bit(bit).unwrap() { return false; }
        }
        true
    }

    fn popcount(&self) -> u64 {
        self.bits.iter().map(|w| Bits::<u64>::new(*w).count_ones() as u64).sum()
    }
}

fn main() {
    // 4096-bit filter, 3 hash functions, target ~5% false-positive rate at ~700 items.
    let mut b = Bloom::new(4096, 3);
    let inserted: Vec<u64> = (0..700u64).map(|i| i.wrapping_mul(2654435761)).collect();
    for &v in &inserted { b.insert(v); }

    let true_positives = inserted.iter().filter(|&&v| b.contains(v)).count();
    println!("Inserted {}, recalled {} ({}% — must be 100%).",
        inserted.len(), true_positives, 100 * true_positives / inserted.len());

    // Probe with 10k values not in the inserted set, count false positives.
    let probes = 10_000u64;
    let mut fp = 0u64;
    for v in 1_000_000..1_000_000 + probes {
        if b.contains(v) { fp += 1; }
    }
    println!("False positives: {} of {} probes ({}%).",
        fp, probes, 100 * fp / probes);
    println!("Filter density: {} of {} bits set ({}%).",
        b.popcount(), b.bit_count, 100 * b.popcount() / b.bit_count as u64);
}
