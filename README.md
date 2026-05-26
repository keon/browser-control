# bitkit

[![Crates.io](https://img.shields.io/crates/v/bitkit.svg)](https://crates.io/crates/bitkit)
[![Docs.rs](https://docs.rs/bitkit/badge.svg)](https://docs.rs/bitkit)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](#license)

> High-performance, width-aware bit manipulation around a single `Bits<T>` newtype.

Common bit-twiddling idioms get a typed, named, checked equivalent:

```rust
// before                                       // after
(addr + align - 1) & !(align - 1)               bitkit::align::up(addr, align)?
(byte >> 4) & 0b1111                            Bits::<u8>::new(byte).extract(4..8)?
(x >> n) & 1 == 1                               Bits::<u32>::new(x).has_bit(n)?
x & x.wrapping_neg()                            Bits::<u32>::new(x).isolate_lowest_set_bit()
((hi as u32) << 16) | (y as u32)                bitkit::morton::encode_2d(hi, y)
unsafe { _pext_u32(x, mask) }                   Bits::<u32>::new(x).gather(mask.into())
```

## Why bitkit

- **Safer.** No panics on out-of-range bit indexes, no silent
  truncation. Every fallible operation returns `Result<_, BitError>`,
  so you can `?`-propagate naturally.
- **Faster on modern hardware.** On x86-64 with BMI2 (Intel Haswell or
  newer, AMD Zen 3 or newer), `gather` and `scatter` compile down to a
  single instruction (`PEXT` / `PDEP`); on other targets they fall back
  to a portable, safe loop. Performance is measured at parity with raw
  `unsafe` BMI2 wrappers — see [BENCHMARKS.md](BENCHMARKS.md).
- **Easier to read in review.** Names like `isolate_lowest_set_bit`
  describe what the trick does instead of obscuring it as `x & -x`.
- **Embedded-friendly.** `#![no_std]`, zero runtime dependencies, MSRV
  Rust 1.74. The only `unsafe` in the crate lives in one isolated
  internal module that uses BMI2 intrinsics.
- **Built-in spatial indexing.** The `bitkit::morton` module ships
  Morton (Z-order) curve encode and decode for quadtrees, GPU textures,
  and range queries — **83× faster** than the `morton-encoding` crate
  on BMI2 hardware and 8.4× faster on portable SWAR.

## Install

```toml
[dependencies]
bitkit = "3"
```

For `no_std`:

```toml
bitkit = { version = "3", default-features = false }
```

## What it looks like

### Bit fields

Extract, insert, or replace a contiguous run of bits using a Rust
`Range` (or a `(start, end)` tuple):

```rust
use bitkit::prelude::*;

let header = Bits::<u8>::new(0b0100_0101);
let version = header.extract(4..8)?; // -> Bits::<u8>::new(0b0100)
let ihl     = header.extract(0..4)?; // -> Bits::<u8>::new(0b0101)

let packed  = Bits::<u8>::new(0)
    .insert(4..8, 4)?
    .insert(0..4, 5)?;               // -> 0b0100_0101
```

### Iterators

Walk the set bits, the zero bits, or every subset of a mask:

```rust
let mask = Bits::<u32>::new(0b1011);
let indexes: Vec<u32> = mask.set_bits().collect();   // [0, 1, 3]

// Every subset of a bitmask, in 2^popcount(mask) steps:
for subset in mask.submasks() { /* ... */ }
```

### Binary protocol parsing

`bitkit::bytes` reads endian-aware integers from a `&[u8]`; `Bits::extract`
peels off the fields:

```rust
let buf = [0x12, 0x34, 0x56, 0x78];
let len    = bitkit::bytes::read_u16_be(&buf)?;        // 0x1234
let flags  = Bits::<u8>::new(buf[0]).extract(5..8)?;   // top 3 bits
```

### Hardware-accelerated bit gather

Pull a non-contiguous set of bits out of a word and pack them together,
in one instruction on capable hardware:

```rust
let v    = Bits::<u32>::new(0b1011_0101);
let mask = Bits::<u32>::new(0b1001_0101);
let packed = v.gather(mask);   // -> Bits::<u32>::new(0b1111)
```

### Morton (Z-order) curves

Interleave the bits of two or three coordinates into a single integer
key — the standard trick behind quadtrees and Z-order indexing:

```rust
let z = bitkit::morton::encode_2d(1234, 5678);          // -> 37247404
assert_eq!(bitkit::morton::decode_2d(z), (1234, 5678));
```

### Flag sets

A lightweight wrapper for ad-hoc flag manipulation where the bit
positions come from data rather than from a fixed name list:

```rust
const READ: u32 = 1 << 0;
const WRITE: u32 = 1 << 1;

let mut f = Flags::<u32>::empty();
f.enable(READ | WRITE);
assert!(f.has_all(READ | WRITE));
```

## What this crate is not

- **Not [`bitvec`](https://crates.io/crates/bitvec).** It does not store
  arbitrary-length bit sequences or expose bit-precise references.
- **Not [`bitflags`](https://crates.io/crates/bitflags).** `Flags<T>`
  is for *ad-hoc* manipulation where bit positions are data; for fixed
  named sets, use `bitflags`.
- **Not a serialization framework.** For declarative binary parsing, use
  [`deku`](https://crates.io/crates/deku) on top.

## Modules

| module                  | purpose                                                                       |
|-------------------------|-------------------------------------------------------------------------------|
| [`Bits<T>`]             | The primary type: bit queries, masks, fields, popcount, gather/scatter.       |
| [`Flags<T>`]            | Generic flag-set newtype.                                                     |
| [`align`]               | Power-of-two alignment over `usize`.                                          |
| [`bytes`]               | Read/write integers from `&[u8]` with explicit endianness.                    |
| [`morton`]              | Z-order curve encode / decode (2D, 3D).                                       |
| [`mod@format`]          | Allocation-free grouped-binary `Display`.                                     |
| [`explain`] (feature)   | Educational metadata for common bit hacks.                                    |

## Feature flags

| flag             | default | purpose                                                              |
|------------------|---------|----------------------------------------------------------------------|
| `std`            | yes     | `std::error::Error` for `BitError`.                                  |
| `alloc`          | yes     | Allocation-backed helpers in optional modules.                       |
| `explain`        | no      | Educational metadata for common bit hacks.                           |
| `runtime-detect` | no      | Use `is_x86_feature_detected!` at runtime to pick the BMI2 path.     |

## Examples

```sh
cargo run --example bit_tricks           # classic bit-twiddling tricks
cargo run --example ipv4_header          # parse real IPv4 header bit fields
cargo run --example permissions          # Unix rwx permissions via Flags<u16>
cargo run --example morton_code          # naive Z-order curve via bit interleaving
cargo run --example morton_pdep          # PDEP-accelerated Morton via bitkit::morton
cargo run --example subset_dp            # subset enumeration with Bits::submasks
cargo run --example chess_knight_moves   # bitboard knight-attack generation
cargo run --example bloom_filter         # Bloom filter on a Vec<u64> bitmap
cargo run --example gray_code            # reflected binary (Gray) codes
cargo run --example rgb_pack             # RGBA8888 / RGB565 color packing
cargo run --example hamming              # Hamming weight + distance + nearest-neighbor
```

## Performance

| benchmark                                            | x86 BMI2     | ARM SWAR     |
|------------------------------------------------------|-------------:|-------------:|
| `Bits::gather` / `scatter` vs `bitintr` (raw unsafe) | tied         | 1.6× faster  |
| `bitkit::morton::encode_2d` vs `morton-encoding`     | **83× faster** | **8.4× faster** |
| `Bits::extract`, `Bits::count_ones` vs inline        | tied         | tied         |
| `Bits::set_bit` vs `bitvec::set`                     | 2.0× faster  | 1.6× faster  |

Full table, methodology, and reproduction recipe in
[BENCHMARKS.md](BENCHMARKS.md).

## License

MIT — see [LICENSE](LICENSE) or
https://opensource.org/licenses/MIT.
