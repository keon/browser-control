# bitkit

[![Crates.io](https://img.shields.io/crates/v/bitkit.svg)](https://crates.io/crates/bitkit)
[![Docs.rs](https://docs.rs/bitkit/badge.svg)](https://docs.rs/bitkit)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](#license)

> **Bit manipulation that reads like the comment you'd have to write next to it.**

```rust
// before                                       // after
(addr + align - 1) & !(align - 1)               bitkit::align::up(addr, align)?
(byte >> 4) & 0b1111                            Bits::<u8>::new(byte).extract(4..8)?
(x >> n) & 1 == 1                               Bits::<u32>::new(x).has_bit(n)?
x & x.wrapping_neg()                            Bits::<u32>::new(x).isolate_lowest_set_bit()
((hi as u32) << 16) | (y as u32)                bitkit::morton::encode_2d(hi, y)
unsafe { _pext_u32(x, mask) }                   Bits::<u32>::new(x).gather(mask.into())
```

One `Bits<T>` newtype. One `Result<_, BitError>` for everything fallible.
No `unsafe` in your code. No panics on bad indices.  And — on x86-64 BMI2 —
the `gather` / `scatter` methods above lower to a **single hardware instruction**.

## Highlights

- **One newtype, one error model, one range convention.** Every bit
  operation is a method on `Bits<T>` returning `Result<_, BitError>`,
  accepting any `Range<u32>` or `(u32, u32)` tuple.
- **Hardware-accelerated `gather` / `scatter`** (PEXT/PDEP on x86-64
  BMI2) with a portable SWAR fallback. The safe wrapper measures within
  **0.2 ns** of the raw unsafe [`bitintr`](https://crates.io/crates/bitintr)
  crate on capable hosts.
- **Morton (Z-order) curves** in a dedicated module — **83× faster** than
  [`morton-encoding`](https://crates.io/crates/morton-encoding) on Intel
  BMI2, **8.4× faster** even on portable SWAR. See [BENCHMARKS.md](BENCHMARKS.md).
- **`no_std`, zero runtime dependencies, no `unsafe`** in the default
  build path (one contained internal module uses BMI2 intrinsics).
- **`const fn` everywhere it can be.**
- **MSRV 1.74.**

## Install

```toml
[dependencies]
bitkit = "3"
```

`no_std`:

```toml
bitkit = { version = "3", default-features = false }
```

## What it looks like

### Bit fields

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

```rust
let mask = Bits::<u32>::new(0b1011);
let indexes: Vec<u32> = mask.set_bits().collect();   // [0, 1, 3]

// All subsets of a bitmask, in O(2^popcount) time
for subset in mask.submasks() { /* ... */ }
```

### Binary protocol parsing

```rust
let buf = [0x12, 0x34, 0x56, 0x78];
let len    = bitkit::bytes::read_u16_be(&buf)?;        // 0x1234
let flags  = Bits::<u8>::new(buf[0]).extract(5..8)?; // top 3 bits
```

### Hardware-accelerated bit gather (PEXT)

```rust
let v    = Bits::<u32>::new(0b1011_0101);
let mask = Bits::<u32>::new(0b1001_0101);
let packed = v.gather(mask);   // -> Bits::<u32>::new(0b1111)
// One PEXT instruction on BMI2; portable SWAR otherwise.
```

### Morton (Z-order) curves

```rust
let z = bitkit::morton::encode_2d(1234, 5678);          // -> 37247404
assert_eq!(bitkit::morton::decode_2d(z), (1234, 5678));
```

### Flag sets

```rust
const READ: u32 = 1 << 0;
const WRITE: u32 = 1 << 1;

let mut f = Flags::<u32>::empty();
f.enable(READ | WRITE);
assert!(f.has_all(READ | WRITE));
```

## What this crate is not

- **Not [`bitvec`](https://crates.io/crates/bitvec).** It doesn't store
  arbitrary-length bit sequences or expose bit-precise references.
- **Not [`bitflags`](https://crates.io/crates/bitflags).** `Flags<T>`
  is for *ad-hoc* manipulation where bit positions are data; for fixed
  named sets, use `bitflags`.
- **Not a serialization framework.** For declarative binary parsing, use
  [`deku`](https://crates.io/crates/deku) on top.

## Modules

| module                  | purpose                                                                       |
|-------------------------|-------------------------------------------------------------------------------|
| [`Bits<T>`]             | The primary type: bit ops, masks, fields, popcount, gather/scatter.           |
| [`Flags<T>`]            | Generic flag-set newtype.                                                     |
| [`align`]               | Power-of-two alignment over `usize`.                                          |
| [`bytes`]               | Read/write integers from `&[u8]` with explicit endianness.                    |
| [`morton`]              | Z-order curve encode / decode (2D, 3D).                                       |
| [`format`]              | Allocation-free grouped-binary `Display`.                                     |
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
cargo run --example bit_tricks           # classic awesome-bits / Bit Twiddling Hacks tricks
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
| `bitkit::morton::encode_2d` vs `morton-encoding` crate | **83× faster** | **8.4× faster** |
| `Bits::extract`, `Bits::count_ones` vs inline        | tied         | tied         |
| `Bits::set_bit` vs `bitvec::set`                     | 2.0× faster  | 1.6× faster  |

Full table, methodology, and reproduction recipe in
[BENCHMARKS.md](BENCHMARKS.md).

## License

MIT — see [LICENSE](LICENSE) or
https://opensource.org/licenses/MIT.
