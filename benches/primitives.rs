//! Microbenchmarks: zero-cost-abstraction sanity check.
//!
//! These benches exist to verify that `Bits::set_bit`, `Bits::extract`,
//! and `Bits::count_ones` compile to the same code as inline expressions
//! — i.e. the wrapper is free at runtime. We are not trying to *beat*
//! anything here; ties are the goal.
//!
//! `bitvec` is included as a sanity check for comparable operations on
//! a fundamentally different data structure (bit-slice over storage).

use bitvec::prelude::*;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

use bitkit::Bits;

fn bench_set_bit(c: &mut Criterion) {
    let idxs: Vec<u32> = (0..1024u32).map(|i| i % 32).collect();

    let mut grp = c.benchmark_group("set_bit_u32");

    grp.bench_function("bits-rs", |b| {
        b.iter(|| {
            let mut x = Bits::<u32>::new(0);
            for &i in &idxs { x = x.set_bit(i).unwrap(); }
            black_box(x)
        })
    });

    grp.bench_function("inline", |b| {
        b.iter(|| {
            let mut x = 0u32;
            for &i in &idxs { x |= 1u32 << i; }
            black_box(x)
        })
    });

    grp.bench_function("bitvec", |b| {
        b.iter(|| {
            let mut bv = bitvec![u32, Lsb0; 0; 32];
            for &i in &idxs { bv.set(i as usize, true); }
            black_box(bv)
        })
    });

    grp.finish();
}

fn bench_extract(c: &mut Criterion) {
    let xs: Vec<u32> = (0..1024u32).map(|i| i.wrapping_mul(0x9E37_79B1)).collect();

    let mut grp = c.benchmark_group("extract_4_8_u32");

    grp.bench_function("bits-rs", |b| {
        b.iter(|| {
            let mut acc = 0u32;
            for &x in &xs {
                acc ^= Bits::<u32>::new(x).extract_indices(4, 8).unwrap().get();
            }
            black_box(acc)
        })
    });

    grp.bench_function("inline", |b| {
        b.iter(|| {
            let mut acc = 0u32;
            for &x in &xs { acc ^= (x >> 4) & 0xF; }
            black_box(acc)
        })
    });

    grp.finish();
}

fn bench_count_ones(c: &mut Criterion) {
    let xs: Vec<u32> = (0..1024u32).map(|i| i.wrapping_mul(0x9E37_79B1)).collect();

    let mut grp = c.benchmark_group("count_ones_u32");

    grp.bench_function("bits-rs", |b| {
        b.iter(|| {
            let mut acc: u32 = 0;
            for &x in &xs { acc = acc.wrapping_add(Bits::<u32>::new(x).count_ones()); }
            black_box(acc)
        })
    });

    grp.bench_function("primitive", |b| {
        b.iter(|| {
            let mut acc: u32 = 0;
            for &x in &xs { acc = acc.wrapping_add(x.count_ones()); }
            black_box(acc)
        })
    });

    grp.finish();
}

criterion_group!(benches, bench_set_bit, bench_extract, bench_count_ones);
criterion_main!(benches);
