//! Microbenchmarks: `Bits::gather` / `Bits::scatter` vs incumbents.
//!
//! Run on x86_64 with BMI2 to see the hardware path:
//!     RUSTFLAGS="-C target-cpu=native" cargo bench --bench gather_scatter

use criterion::{black_box, criterion_group, criterion_main, Criterion};

use bits::Bits;
use bitintr::{Pdep, Pext};

// SWAR baseline (taken from src/accel.rs portable fallback, semantics-equivalent).
#[inline(never)]
fn pext_swar_u32(x: u32, mut mask: u32) -> u32 {
    let mut res = 0u32;
    let mut bb = 1u32;
    while mask != 0 {
        let lowest = mask & mask.wrapping_neg();
        if (x & lowest) != 0 { res |= bb; }
        mask ^= lowest;
        bb <<= 1;
    }
    res
}
#[inline(never)]
fn pdep_swar_u32(x: u32, mut mask: u32) -> u32 {
    let mut res = 0u32;
    let mut bb = 1u32;
    while mask != 0 {
        let lowest = mask & mask.wrapping_neg();
        if (x & bb) != 0 { res |= lowest; }
        mask ^= lowest;
        bb <<= 1;
    }
    res
}

fn bench_pext_u32(c: &mut Criterion) {
    let xs: Vec<u32> = (0..256u32).map(|i| i.wrapping_mul(0x9E37_79B1)).collect();
    let mask: u32 = 0xF0F0_F0F0;

    let mut grp = c.benchmark_group("pext_u32");

    grp.bench_function("bits-rs", |b| {
        b.iter(|| {
            let mut acc = 0u32;
            for &x in &xs {
                acc ^= black_box(Bits::<u32>::new(x).gather(Bits::new(mask))).get();
            }
            acc
        })
    });
    grp.bench_function("bitintr", |b| {
        b.iter(|| {
            let mut acc = 0u32;
            for &x in &xs {
                acc ^= black_box(x.pext(mask));
            }
            acc
        })
    });
    grp.bench_function("swar", |b| {
        b.iter(|| {
            let mut acc = 0u32;
            for &x in &xs {
                acc ^= black_box(pext_swar_u32(x, mask));
            }
            acc
        })
    });

    grp.finish();
}

fn bench_pdep_u32(c: &mut Criterion) {
    let xs: Vec<u32> = (0..256u32).map(|i| i.wrapping_mul(0x9E37_79B1)).collect();
    let mask: u32 = 0xF0F0_F0F0;

    let mut grp = c.benchmark_group("pdep_u32");
    grp.bench_function("bits-rs", |b| {
        b.iter(|| {
            let mut acc = 0u32;
            for &x in &xs {
                acc ^= black_box(Bits::<u32>::new(x).scatter(Bits::new(mask))).get();
            }
            acc
        })
    });
    grp.bench_function("bitintr", |b| {
        b.iter(|| {
            let mut acc = 0u32;
            for &x in &xs {
                acc ^= black_box(x.pdep(mask));
            }
            acc
        })
    });
    grp.bench_function("swar", |b| {
        b.iter(|| {
            let mut acc = 0u32;
            for &x in &xs {
                acc ^= black_box(pdep_swar_u32(x, mask));
            }
            acc
        })
    });
    grp.finish();
}

// ---- Morton 2D encode -------------------------------------------------

const X_LANES: u32 = 0x5555_5555;
const Y_LANES: u32 = 0xAAAA_AAAA;

#[inline]
fn morton_bits_rs(x: u16, y: u16) -> u32 {
    (Bits::<u32>::new(x as u32).scatter(Bits::new(X_LANES))
     | Bits::<u32>::new(y as u32).scatter(Bits::new(Y_LANES))).get()
}
#[inline(never)]
fn morton_bit_by_bit(x: u16, y: u16) -> u32 {
    let mut z = 0u32;
    for i in 0..16 {
        z |= ((x as u32 >> i) & 1) << (2 * i);
        z |= ((y as u32 >> i) & 1) << (2 * i + 1);
    }
    z
}

fn bench_morton(c: &mut Criterion) {
    let pts: Vec<(u16, u16)> = (0..1024u32).map(|i| (i as u16, (i.wrapping_mul(31)) as u16)).collect();

    let mut grp = c.benchmark_group("morton2d_encode");

    grp.bench_function("bits-rs-pdep", |b| {
        b.iter(|| {
            let mut acc = 0u32;
            for &(x, y) in &pts { acc ^= black_box(morton_bits_rs(x, y)); }
            acc
        })
    });

    grp.bench_function("bit-by-bit", |b| {
        b.iter(|| {
            let mut acc = 0u32;
            for &(x, y) in &pts { acc ^= black_box(morton_bit_by_bit(x, y)); }
            acc
        })
    });

    grp.bench_function("morton-encoding-crate", |b| {
        b.iter(|| {
            let mut acc = 0u32;
            for &(x, y) in &pts {
                let z: u32 = morton_encoding::morton_encode([x, y]);
                acc ^= black_box(z);
            }
            acc
        })
    });

    grp.finish();
}

criterion_group!(benches, bench_pext_u32, bench_pdep_u32, bench_morton);
criterion_main!(benches);
