# Benchmarks

Comparative measurements of `bits-rs` v3.2.1 against incumbent Rust
crates and hand-written inline baselines. Numbers were collected with
[Criterion](https://github.com/bheisler/criterion.rs).

Two hosts, run on the same v3.2.1 source:

| host | machine | CPU | BMI2 | role |
|---|---|---|---|---|
| ARM | Apple M-series, macOS | Apple Silicon | ✗ | portable SWAR path |
| x86 | GCP `n2-standard-2`, Ubuntu 22.04 | Intel Xeon @ 2.8 GHz (Cascade Lake) | ✓ | hardware path |

x86 numbers were collected with `RUSTFLAGS="-C target-cpu=native"`.
Each benchmark loops a small kernel over 256–1024 inputs to amortize
measurement overhead. Reported times are criterion's median across 100
samples.

## 1. Gather / scatter — `bits-rs` vs `bitintr` vs SWAR

256 ops/sample.

| op           | host | bits-rs | bitintr | SWAR    | bits-rs vs bitintr | bits-rs vs SWAR |
|--------------|------|--------:|--------:|--------:|--------------------:|----------------:|
| **PEXT u32** | x86  | **88.7 ns** | 88.6 ns | 3060 ns | tied (within 0.1 ns) | **34.5× faster** |
| PEXT u32     | ARM  | 1.91 µs | 3.14 µs | 1.96 µs | 1.6× faster          | tied             |
| **PDEP u32** | x86  | **89.3 ns** | 89.4 ns | 3024 ns | tied (within 0.2 ns) | **33.9× faster** |
| PDEP u32     | ARM  | 1.98 µs | 3.00 µs | 1.84 µs | 1.5× faster          | within noise     |

On BMI2 hardware the `bits-rs` safe wrapper matches `bitintr`'s raw
unsafe API to within measurement noise — both lower to a single
`PEXT` / `PDEP` instruction. On non-BMI2 hardware the SWAR fallback
runs ~34× slower than the hardware instruction but still edges out
`bitintr`'s own portable fallback by ~1.6×.

## 2. Morton (Z-order) 2D encode

1024 points/sample.

| host | implementation                | median time | per-point | vs `morton-encoding` |
|------|-------------------------------|------------:|----------:|---------------------:|
| **x86 BMI2** | **`bits::morton::encode_2d`** | **690 ns**  | 0.67 ns   | **83×**             |
| x86 BMI2     | naive 16-iter loop            | 5.82 µs     | 5.69 ns   | 9.9×                 |
| x86 BMI2     | `morton-encoding` crate       | 57.46 µs    | 56.1 ns   | 1.0× (baseline)      |
| **ARM SWAR** | **`bits::morton::encode_2d`** | **2.02 µs** | 1.97 ns   | **8.4×**             |
| ARM SWAR     | naive 16-iter loop            | 2.32 µs     | 2.27 ns   | 7.4×                 |
| ARM SWAR     | `morton-encoding` crate       | 17.08 µs    | 16.7 ns   | 1.0×                 |

The non-BMI2 advantage comes from skipping the per-bit interleaving
loop in favor of a SWAR PDEP-equivalent. The BMI2 advantage is the
single-instruction PDEP path.

## 3. Primitives — zero-cost-abstraction verification

| benchmark | host | bits-rs | inline / primitive | bitvec | margin |
|-----------|------|--------:|-------------------:|-------:|--------|
| `extract(4..8)`   | x86 | **37.9 ns** | 37.9 ns | —       | tied                 |
| `extract(4..8)`   | ARM | **53.7 ns** | 49.8 ns | —       | inline 8% faster (noise) |
| `count_ones`      | x86 | 267 ns      | 268 ns  | —       | tied                 |
| `count_ones`      | ARM | **38.9 ns** | 43.8 ns | —       | tied (within noise)  |
| `set_bit` × 1024  | x86 | 615 ns      | 49 ns ⚠ | 1245 ns | wrapper 2× faster than `bitvec` |
| `set_bit` × 1024  | ARM | 306 ns      | 52 ns ⚠ | 490 ns  | wrapper 1.6× faster than `bitvec` |
| `popcount_sum_1M` | x86 | 367 µs      | 354 µs  | —       | tied (within noise)  |

**⚠ `set_bit` inline baseline is a benchmarking artifact.** LLVM
constant-folds the inline loop over a `Vec` of known-bounded indices
(`i % 32`) into something close to `x = u32::MAX`, which it cannot see
through `Bits::set_bit`'s bounds-check `Result`. In real call sites
with opaque indices both versions emit identical code.

## 4. Medium-complexity workloads

### TSP-15 Held–Karp dynamic programming

`for x in iter` (standard) vs `iter.for_each(|x| ...)` (fast path) vs
hand-written `while m != 0 { trailing_zeros; m &= m - 1 }` loop.

| variant                                          | x86 BMI2     | ARM SWAR     |
|--------------------------------------------------|-------------:|-------------:|
| **bits-rs (standard `for x in iter`)**           | 2.08 ms      | **1.76 ms**  |
| inline (raw `while m != 0` ops)                  | **1.36 ms**  | 1.92 ms      |
| **bits-rs (`iter.for_each(\|x\|...)` fast path)**| **1.56 ms**  | 2.14 ms      |

On x86 BMI2 the standard iterator is 1.53× slower than the raw inline
loop; the `for_each` fast path (added in v3.2) closes 75% of that gap,
bringing it down to 1.15× slower. On ARM the standard iterator is
already faster than both alternatives — the iterator shape with the
best codegen is platform-dependent.

### IPv4 header parse × 10 000

| variant                                          | x86 BMI2 | ARM SWAR |
|--------------------------------------------------|---------:|---------:|
| bits-rs (`bytes::read_*` + `Bits::extract`)      | 16.66 µs | **4.22 µs** |
| inline (`u32::from_be_bytes` + shift/and)        | 16.64 µs | 5.24 µs  |
| margin                                           | tied     | bits-rs 1.24× faster |

### Sparse 4-flag extract × 10 000

| variant                                  | x86 BMI2 | ARM SWAR |
|------------------------------------------|---------:|---------:|
| bits-rs (single `gather` / PEXT)         | 2.99 µs  | 2.29 µs  |
| inline (4× `shift+and`)                  | **1.48 µs** | **1.62 µs** |
| margin                                   | inline 2.0× faster | inline 1.4× faster |

This is a hardware property of PEXT (3-cycle latency on Skylake-class
Intel), not a library cost: four independent inline shifts pipeline
through fewer cycles than a single PEXT for small popcount masks. PEXT
wins on larger gathers — roughly ≥8 non-contiguous bits at once.

## 5. Cross-platform summary

| benchmark | ARM (no BMI2) | x86 BMI2 | notes |
|---|---:|---:|---|
| Morton 2D encode (single point) | 2.02 µs | **0.69 µs** | 2.9× speedup |
| Morton vs `morton-encoding` | 8.4× faster | **83× faster** | headline figure |
| PEXT u32 (256 ops) | 1.91 µs | **89 ns** | 21× speedup |
| PDEP u32 (256 ops) | 1.98 µs | **89 ns** | 22× speedup |
| gather/scatter vs `bitintr` | 1.6× ahead (SWAR vs SWAR) | dead tie (both PEXT) | as expected |
| TSP-15 (best variant) | 1.76 ms (standard iter) | 1.36 ms (inline) | iterator path-dependent |
| TSP-15 (`for_each` fast path) | 2.14 ms | **1.56 ms** | helps on x86, hurts on ARM |
| IPv4 parse 10K | bits-rs 1.24× faster | tied | both well-optimized on x86 |
| `extract`, `count_ones` | tied | tied | zero-cost everywhere |
| Sparse 4-flag extract | inline 1.4× faster | inline 2.0× faster | hardware PEXT latency |

## Reproducing the numbers

```sh
# Portable SWAR path (any platform).
cargo bench

# Hardware BMI2 path (x86_64 Haswell+/Zen 3+ only).
RUSTFLAGS="-C target-cpu=native" cargo bench

# Single comparison group:
cargo bench --bench gather_scatter -- morton2d
cargo bench --bench workloads -- tsp15
```

## Notes and caveats

- Numbers were collected on quiet machines but **not** with frequency
  scaling locked or process pinning. Treat ±5% as ambient noise.
- The x86 host was a GCP `n2-standard-2` (~$0.097/hr; total spend for
  this bench sweep was ~$0.01; the instance was deleted immediately
  after the run).
- No claim is made about beating hardware. The reported wins are
  against other libraries (`morton-encoding`, `bitintr`'s portable
  fallback, `bitvec` for unrelated workloads) and against hand-written
  inline code in specific kernel shapes.
- The `set_bit` "inline" baseline is a constant-folding artifact,
  flagged inline in the table.
- The `sparse_extract` result reflects Skylake PEXT latency, not a
  library issue.
