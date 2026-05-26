//! Reflected binary (Gray) codes. Consecutive Gray codes differ in
//! exactly one bit — useful for rotary encoders, Karnaugh maps, and
//! incremental subset DPs.
//!
//! `to_gray(n)`   = n ^ (n >> 1)
//! `from_gray(g)` = g ^ (g >> 1) ^ (g >> 2) ^ ... (XOR all higher bits)
//!
//! Run: `cargo run --example gray_code`

fn to_gray(n: u32) -> u32 { n ^ (n >> 1) }

fn from_gray(mut g: u32) -> u32 {
    g ^= g >> 16;
    g ^= g >> 8;
    g ^= g >> 4;
    g ^= g >> 2;
    g ^= g >> 1;
    g
}

fn main() {
    println!(" n   binary  gray  differs in");
    println!("---  ------  ----  ----------");
    let mut prev = 0u32;
    for n in 0..16u32 {
        let g = to_gray(n);
        let differs = if n == 0 {
            String::from("-")
        } else {
            format!("bit {}", (g ^ prev).trailing_zeros())
        };
        println!(" {n:>2}    {n:04b}  {g:04b}  {differs}");
        prev = g;
    }

    for n in [0u32, 1, 42, 1000, 0xCAFE_BABE, u32::MAX] {
        assert_eq!(from_gray(to_gray(n)), n);
    }
    println!("\nRound-trip verified for {{0, 1, 42, 1000, 0xCAFE_BABE, u32::MAX}}.");
}
