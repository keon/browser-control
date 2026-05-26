//! Subset-enumeration DP using `Bits::submasks`.
//!
//! Problem: given a set of `n` items each with a weight, find the
//! lightest subset whose weights sum to exactly `target`. Brute-force
//! over all 2^n subsets, but using the `submasks` iterator over a single
//! starting mask lets us enumerate the subsets of *that* mask cleanly.
//!
//! Run: `cargo run --example subset_dp`

use bits::{format::grouped_binary, prelude::*};

fn min_subset_for_sum(weights: &[u32], target: u32) -> Option<u8> {
    assert!(weights.len() <= 8);
    let universe = Bits::<u8>::new((1u8 << weights.len()).wrapping_sub(1));

    let mut best: Option<u8> = None;
    let mut best_count = u32::MAX;

    for subset in universe.submasks() {
        let sum: u32 = subset.set_bits().map(|i| weights[i as usize]).sum();
        if sum == target {
            let count = subset.count_ones();
            if count < best_count {
                best_count = count;
                best = Some(subset.get());
            }
        }
    }
    best
}

fn main() {
    let weights = [3u32, 5, 7, 8, 11];
    let target = 16;

    match min_subset_for_sum(&weights, target) {
        Some(mask) => {
            let m = Bits::<u8>::new(mask);
            println!("weights = {:?}", weights);
            println!("target  = {target}");
            println!("mask    = {}", grouped_binary(m, 4));
            let picked: Vec<u32> = m.set_bits().map(|i| weights[i as usize]).collect();
            let sum: u32 = picked.iter().sum();
            println!("picked  = {:?} ({} items, sum = {})", picked, picked.len(), sum);
        }
        None => println!("no subset of {:?} sums to {target}", weights),
    }
}
