//! Bitboard knight-move generation — the classic chess engine pattern.
//!
//! A chess board is 64 squares, indexed a1=0, b1=1, ..., h8=63. A
//! "bitboard" stores piece positions as a `u64` (one bit per square).
//! For each starting square, the set of reachable squares for a knight
//! is a precomputable mask.
//!
//! Run: `cargo run --example chess_knight_moves`

use bits::{format::grouped_binary, prelude::*};

fn knight_attacks(sq: u32) -> Bits<u64> {
    // A knight at square `sq` can jump in 8 directions; we mask each
    // shift against file-boundary masks to keep moves on the board.
    let knight = Bits::<u64>::new(1u64 << sq);
    let not_a  = Bits::<u64>::new(!0x0101_0101_0101_0101); // not file a
    let not_h  = Bits::<u64>::new(!0x8080_8080_8080_8080); // not file h
    let not_ab = Bits::<u64>::new(!0x0303_0303_0303_0303); // not files a or b
    let not_gh = Bits::<u64>::new(!0xC0C0_C0C0_C0C0_C0C0); // not files g or h

    let nne = Bits::<u64>::new((knight & not_h).get() << 17);
    let nnw = Bits::<u64>::new((knight & not_a).get() << 15);
    let nee = Bits::<u64>::new((knight & not_gh).get() << 10);
    let nww = Bits::<u64>::new((knight & not_ab).get() << 6);
    let sse = Bits::<u64>::new((knight & not_h).get() >> 15);
    let ssw = Bits::<u64>::new((knight & not_a).get() >> 17);
    let see = Bits::<u64>::new((knight & not_gh).get() >> 6);
    let sww = Bits::<u64>::new((knight & not_ab).get() >> 10);

    nne | nnw | nee | nww | sse | ssw | see | sww
}

fn render(b: Bits<u64>) -> String {
    let mut out = String::new();
    for rank in (0..8).rev() {
        out.push_str(&format!("{}  ", rank + 1));
        for file in 0..8 {
            let sq = rank * 8 + file;
            out.push(if b.has_bit(sq).unwrap() { '*' } else { '.' });
            out.push(' ');
        }
        out.push('\n');
    }
    out.push_str("   a b c d e f g h\n");
    out
}

fn main() {
    // Knight on d4 (file d = 3, rank 4 → square 3 + 3*8 = 27).
    let d4_attacks = knight_attacks(27);
    println!("Knight on d4 attacks {} squares:", d4_attacks.count_ones());
    println!("{}", render(d4_attacks));

    // Knight on a1: only 2 legal moves (b3, c2).
    let a1_attacks = knight_attacks(0);
    println!("Knight on a1 attacks {} squares:", a1_attacks.count_ones());
    println!("{}", render(a1_attacks));

    // Maximum reach is from squares in the central 4x4.
    let mut max = 0u32;
    let mut best_sq = 0u32;
    for sq in 0..64 {
        let n = knight_attacks(sq).count_ones();
        if n > max { max = n; best_sq = sq; }
    }
    println!("Max knight reach: {} squares (from square index {}).", max, best_sq);
    println!("Raw bitboard: {}", grouped_binary(knight_attacks(27), 8));
}
