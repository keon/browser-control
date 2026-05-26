//! Medium-complexity workload benchmarks.
//!
//! Each benchmark exercises a realistic multi-step kernel rather than a
//! single arithmetic op:
//!
//! * `tsp15` — Held–Karp travelling-salesman DP on 15 cities, driven by
//!   the `submasks` iterator.
//! * `ipv4_parse_10k` — parse 10 000 IPv4 headers byte-for-byte.
//! * `sparse_extract` — pull 8 non-contiguous bit fields from a packed
//!   32-bit word.
//! * `popcount_sum_1M` — sum the population count of 1 million u64s.

#![allow(clippy::identity_op, clippy::needless_range_loop, clippy::erasing_op, clippy::unusual_byte_groupings)]

use criterion::{black_box, criterion_group, criterion_main, Criterion};

use bits::{bytes, Bits};

// =========================================================================
// 1. Held–Karp TSP DP over a 15-city distance matrix.
// =========================================================================
//
// dp[mask][last] = shortest path visiting the cities in `mask`, ending
// at `last`. Recurrence enumerates submasks of `mask` excluding `last`
// to find the predecessor. We compare two equivalent implementations:
//
// * `tsp_bitsrs` — uses `Bits::set_bits` to iterate predecessors and
//                   `Bits::has_bit` for set membership.
// * `tsp_inline` — uses bitwise ops directly.

const N: usize = 15;

fn random_distance_matrix(seed: u64) -> [[u32; N]; N] {
    let mut d = [[0u32; N]; N];
    let mut s = seed;
    for i in 0..N {
        for j in 0..N {
            if i == j { continue; }
            s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            d[i][j] = (s >> 33) as u32 % 1000 + 1;
        }
    }
    d
}

fn tsp_bitsrs(dist: &[[u32; N]; N]) -> u32 {
    let states = 1usize << N;
    let mut dp = vec![u32::MAX; states * N];
    dp[(1usize) * N + 0] = 0;

    for mask in 1u32..states as u32 {
        let mbits = Bits::<u32>::new(mask);
        if !mbits.has_bit(0).unwrap() { continue; }
        for last in mbits.set_bits() {
            if last == 0 && mask != 1 { continue; }
            let cur = dp[mask as usize * N + last as usize];
            if cur == u32::MAX { continue; }
            // Extend to every city `next` not yet in `mask`.
            let unvisited = !mask & ((1u32 << N) - 1);
            for next in Bits::<u32>::new(unvisited).set_bits() {
                let new_mask = mask | (1u32 << next);
                let cand = cur.saturating_add(dist[last as usize][next as usize]);
                let slot = &mut dp[new_mask as usize * N + next as usize];
                if cand < *slot { *slot = cand; }
            }
        }
    }
    let full = (1u32 << N) - 1;
    let mut best = u32::MAX;
    for last in 1..N {
        let val = dp[full as usize * N + last].saturating_add(dist[last][0]);
        if val < best { best = val; }
    }
    best
}

fn tsp_inline(dist: &[[u32; N]; N]) -> u32 {
    let states = 1usize << N;
    let mut dp = vec![u32::MAX; states * N];
    dp[1 * N + 0] = 0;
    for mask in 1u32..states as u32 {
        if mask & 1 == 0 { continue; }
        let mut m = mask;
        while m != 0 {
            let last = m.trailing_zeros();
            m &= m - 1;
            if last == 0 && mask != 1 { continue; }
            let cur = dp[mask as usize * N + last as usize];
            if cur == u32::MAX { continue; }
            let mut un = !mask & ((1u32 << N) - 1);
            while un != 0 {
                let next = un.trailing_zeros();
                un &= un - 1;
                let new_mask = mask | (1u32 << next);
                let cand = cur.saturating_add(dist[last as usize][next as usize]);
                let slot = &mut dp[new_mask as usize * N + next as usize];
                if cand < *slot { *slot = cand; }
            }
        }
    }
    let full = (1u32 << N) - 1;
    let mut best = u32::MAX;
    for last in 1..N {
        let val = dp[full as usize * N + last].saturating_add(dist[last][0]);
        if val < best { best = val; }
    }
    best
}

fn tsp_bitsrs_foreach(dist: &[[u32; N]; N]) -> u32 {
    let states = 1usize << N;
    let mut dp = vec![u32::MAX; states * N];
    dp[(1usize) * N + 0] = 0;
    for mask in 1u32..states as u32 {
        let mbits = Bits::<u32>::new(mask);
        if !mbits.has_bit(0).unwrap() { continue; }
        mbits.set_bits().for_each(|last| {
            if last == 0 && mask != 1 { return; }
            let cur = dp[mask as usize * N + last as usize];
            if cur == u32::MAX { return; }
            let unvisited = !mask & ((1u32 << N) - 1);
            Bits::<u32>::new(unvisited).set_bits().for_each(|next| {
                let new_mask = mask | (1u32 << next);
                let cand = cur.saturating_add(dist[last as usize][next as usize]);
                let slot = &mut dp[new_mask as usize * N + next as usize];
                if cand < *slot { *slot = cand; }
            });
        });
    }
    let full = (1u32 << N) - 1;
    let mut best = u32::MAX;
    for last in 1..N {
        let val = dp[full as usize * N + last].saturating_add(dist[last][0]);
        if val < best { best = val; }
    }
    best
}

fn bench_tsp(c: &mut Criterion) {
    let d = random_distance_matrix(0xDEAD_BEEF_CAFE_BABE);
    let mut grp = c.benchmark_group("tsp15");
    grp.sample_size(20); // each iteration is heavy
    grp.bench_function("bits-rs (submasks/set_bits)", |b| {
        b.iter(|| black_box(tsp_bitsrs(black_box(&d))))
    });
    grp.bench_function("inline (raw ops)", |b| {
        b.iter(|| black_box(tsp_inline(black_box(&d))))
    });
    grp.bench_function("bits-rs (for_each fast path)", |b| {
        b.iter(|| black_box(tsp_bitsrs_foreach(black_box(&d))))
    });
    grp.finish();
}

// =========================================================================
// 2. Parse 10 000 IPv4 headers.
// =========================================================================

const PACKETS: usize = 10_000;

fn make_packets() -> Vec<[u8; 20]> {
    (0..PACKETS).map(|i| {
        let i32_ = i as u32;
        [
            0x45, 0x00, (i32_ >> 8) as u8, i32_ as u8,
            (i32_ >> 0) as u8, (i32_ >> 8) as u8, 0x40, 0x00,
            64, 6, 0xAB, 0xCD,
            192, 168, (i >> 8) as u8, i as u8,
            8, 8, 8, 8,
        ]
    }).collect()
}

#[derive(Default)]
struct ParsedSummary {
    version_sum: u64,
    ihl_sum: u64,
    total_length_sum: u64,
    ttl_sum: u64,
    src_xor: u32,
}

fn parse_bitsrs(pkts: &[[u8; 20]]) -> ParsedSummary {
    let mut s = ParsedSummary::default();
    for hdr in pkts {
        let b0 = Bits::<u8>::new(hdr[0]);
        s.version_sum += b0.extract(4..8).unwrap().get() as u64;
        s.ihl_sum     += b0.extract(0..4).unwrap().get() as u64;
        s.total_length_sum += bytes::read_u16_be(&hdr[2..]).unwrap() as u64;
        s.ttl_sum     += hdr[8] as u64;
        s.src_xor     ^= bytes::read_u32_be(&hdr[12..]).unwrap();
    }
    s
}

fn parse_inline(pkts: &[[u8; 20]]) -> ParsedSummary {
    let mut s = ParsedSummary::default();
    for hdr in pkts {
        s.version_sum += (hdr[0] >> 4) as u64;
        s.ihl_sum     += (hdr[0] & 0xF) as u64;
        s.total_length_sum += u16::from_be_bytes([hdr[2], hdr[3]]) as u64;
        s.ttl_sum     += hdr[8] as u64;
        s.src_xor     ^= u32::from_be_bytes([hdr[12], hdr[13], hdr[14], hdr[15]]);
    }
    s
}

fn bench_ipv4(c: &mut Criterion) {
    let pkts = make_packets();
    let mut grp = c.benchmark_group("ipv4_parse_10k");
    grp.bench_function("bits-rs (bytes::read + Bits::extract)", |b| {
        b.iter(|| black_box(parse_bitsrs(black_box(&pkts))));
    });
    grp.bench_function("inline (shift/and/from_be_bytes)", |b| {
        b.iter(|| black_box(parse_inline(black_box(&pkts))));
    });
    grp.finish();
}

// =========================================================================
// 3. Sparse bit extract — pull 8 non-contiguous fields from a 32-bit word
//    using one gather (PEXT) vs a shift+and chain.
// =========================================================================
//
// Layout of the 32-bit "message":
//   bit 31:    flag A
//   bits 30-24: opcode (7 bits)
//   bit 23:    flag B
//   bits 22-16: tag (7 bits)
//   bit 15:    flag C
//   bits 14-8: payload-hi (7 bits)
//   bit 7:     flag D
//   bits 6-0:  payload-lo (7 bits)
//
// Gather mask 0xFFFF_FFFF returns the raw 32 bits packed (useless), but
// a realistic example wants the 4 flags packed into a nibble. That's
// what `gather(0x8080_8080)` does in one instruction on BMI2.

fn extract_flags_bitsrs(xs: &[u32]) -> u32 {
    let mask = Bits::<u32>::new(0x8080_8080);
    let mut acc = 0u32;
    for &x in xs {
        acc ^= Bits::<u32>::new(x).gather(mask).get();
    }
    acc
}

fn extract_flags_inline(xs: &[u32]) -> u32 {
    let mut acc = 0u32;
    for &x in xs {
        let a = (x >> 31) & 1;
        let b = (x >> 23) & 1;
        let c = (x >> 15) & 1;
        let d = (x >> 7) & 1;
        acc ^= a | (b << 1) | (c << 2) | (d << 3);
    }
    acc
}

fn bench_sparse_extract(c: &mut Criterion) {
    let xs: Vec<u32> = (0..10_000u32).map(|i| i.wrapping_mul(0x9E37_79B1)).collect();
    let mut grp = c.benchmark_group("sparse_extract_10k");
    grp.bench_function("bits-rs (single gather/PEXT)", |b| {
        b.iter(|| black_box(extract_flags_bitsrs(black_box(&xs))))
    });
    grp.bench_function("inline (shift+and×4)", |b| {
        b.iter(|| black_box(extract_flags_inline(black_box(&xs))))
    });
    grp.finish();
}

// =========================================================================
// 4. Popcount sum over 1 M u64s.
// =========================================================================

fn popcount_sum_bitsrs(xs: &[u64]) -> u64 {
    let mut s: u64 = 0;
    for &x in xs { s = s.wrapping_add(Bits::<u64>::new(x).count_ones() as u64); }
    s
}
fn popcount_sum_inline(xs: &[u64]) -> u64 {
    let mut s: u64 = 0;
    for &x in xs { s = s.wrapping_add(x.count_ones() as u64); }
    s
}

fn bench_popcount_sum(c: &mut Criterion) {
    let xs: Vec<u64> = (0..1_000_000u64)
        .map(|i| i.wrapping_mul(0x9E37_79B97F4A_7C15))
        .collect();
    let mut grp = c.benchmark_group("popcount_sum_1M_u64");
    grp.bench_function("bits-rs", |b| b.iter(|| black_box(popcount_sum_bitsrs(black_box(&xs)))));
    grp.bench_function("primitive", |b| b.iter(|| black_box(popcount_sum_inline(black_box(&xs)))));
    grp.finish();
}

criterion_group!(workloads, bench_tsp, bench_ipv4, bench_sparse_extract, bench_popcount_sum);
criterion_main!(workloads);
