//! Classic bit tricks from awesome-bits and Sean Anderson's
//! *Bit Twiddling Hacks*, rewritten through the `bits` crate.
//!
//! Run: `cargo run --example bit_tricks`

#![allow(clippy::manual_swap)]   // the example *demonstrates* the XOR swap trick

use bitkit::{format::grouped_binary, prelude::*};

fn main() -> Result<(), BitError> {
    let line = |label: &str, x: Bits<u32>| {
        println!("{label:<32} {} = {:>10}", grouped_binary(x, 4), x.get());
    };

    println!("== single-bit ops ==");
    let x = Bits::<u32>::new(0b1010);
    line("set bit 0",   x.set_bit(0)?);
    line("unset bit 1", x.clear_bit(1)?);
    line("toggle bit 2", x.toggle_bit(2)?);

    println!("\n== bit hacks ==");
    let y = Bits::<u32>::new(0b1011_0000);
    line("y",                          y);
    line("isolate lowest set",         y.isolate_lowest_set_bit());
    line("clear  lowest set",          y.clear_lowest_set_bit());
    line("isolate lowest zero",        y.isolate_lowest_zero_bit());
    line("set    lowest zero",         y.set_lowest_zero_bit());

    println!("\n== power of two ==");
    for n in [0u32, 1, 5, 8, 1023, 1 << 30] {
        let b = Bits::<u32>::new(n);
        let next = b.next_power_of_two().map(|v| v.get() as i64).unwrap_or(-1);
        println!("  {n:>10}  is_pow2={:<5} next_pow2={next}", b.is_power_of_two());
    }

    println!("\n== popcount & friends ==");
    let z = Bits::<u32>::new(0xCAFE_BABE);
    println!("  ones    = {}",  z.count_ones());
    println!("  zeros   = {}",  z.count_zeros());
    println!("  ctz     = {}",  z.trailing_zeros());
    println!("  clz     = {}",  z.leading_zeros());

    println!("\n== fields ==");
    // IPv4-style: pack version=4, IHL=5 into one byte.
    let byte = Bits::<u8>::new(0)
        .insert(4..8, 4)?    // version
        .insert(0..4, 5)?;   // IHL
    println!("  packed       = 0b{:08b}", byte.get());
    println!("  version      = {}", byte.extract(4..8)?.get());
    println!("  IHL          = {}", byte.extract(0..4)?.get());

    println!("\n== XOR swap (no temporary) ==");
    let (mut a, mut b) = (0xAAu8, 0x55u8);
    println!("  before:  a=0x{a:02X}  b=0x{b:02X}");
    a ^= b; b ^= a; a ^= b;
    println!("  after:   a=0x{a:02X}  b=0x{b:02X}");

    println!("\n== absolute value without branching (i32) ==");
    for n in [-7i32, -1, 0, 3, 42] {
        let mask = n >> 31;              // 0 if positive, -1 if negative
        let abs = (n ^ mask).wrapping_sub(mask);
        println!("  abs({n:>4}) = {abs}");
    }

    println!("\n== min / max without branching (i32) ==");
    let (p, q) = (17i32, 5i32);
    let min = q ^ ((p ^ q) & -((p < q) as i32));
    let max = p ^ ((p ^ q) & -((p < q) as i32));
    println!("  min({p},{q}) = {min}");
    println!("  max({p},{q}) = {max}");

    println!("\n== reverse bits ==");
    let r = Bits::<u32>::new(0b1000_0001_0000_0000_0000_0000_1100_0000);
    let mut bits_rev = 0u32;
    for i in r.set_bits() {
        bits_rev |= 1 << (Bits::<u32>::BITS - 1 - i);
    }
    line("input ",  r);
    line("reverse", Bits::new(bits_rev));

    println!("\n== submasks (enumerate every subset of 0b1011) ==");
    for s in Bits::<u8>::new(0b1011).submasks() {
        println!("  {}", grouped_binary(s, 4));
    }

    Ok(())
}
