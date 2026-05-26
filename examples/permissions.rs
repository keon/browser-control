//! Unix-style permission bits using `Flags<u16>`.
//!
//! Run: `cargo run --example permissions`

use bitkit::Flags;

// rwxrwxrwx mapped onto a u16.
const X_OTHER: u16 = 1 << 0;
const W_OTHER: u16 = 1 << 1;
const R_OTHER: u16 = 1 << 2;
const X_GROUP: u16 = 1 << 3;
const W_GROUP: u16 = 1 << 4;
const R_GROUP: u16 = 1 << 5;
const X_OWNER: u16 = 1 << 6;
const W_OWNER: u16 = 1 << 7;
const R_OWNER: u16 = 1 << 8;

const SETGID: u16 = 1 << 10;
const SETUID: u16 = 1 << 11;

fn render(p: Flags<u16>) -> String {
    let bit = |f, c: char| if p.has(f) { c } else { '-' };
    format!(
        "{}{}{}{}{}{}{}{}{}",
        bit(R_OWNER, 'r'), bit(W_OWNER, 'w'), bit(X_OWNER, 'x'),
        bit(R_GROUP, 'r'), bit(W_GROUP, 'w'), bit(X_GROUP, 'x'),
        bit(R_OTHER, 'r'), bit(W_OTHER, 'w'), bit(X_OTHER, 'x'),
    )
}

fn main() {
    let mut p = Flags::<u16>::empty();

    // Start with 0644.
    p.enable(R_OWNER | W_OWNER | R_GROUP | W_GROUP | R_OTHER);
    println!("{:04o}  {}  initial", p.bits(), render(p));

    // chmod +x for everyone.
    p.enable(X_OWNER | X_GROUP | X_OTHER);
    println!("{:04o}  {}  after chmod +x", p.bits(), render(p));

    // chmod g-w (group write off).
    p.disable(W_GROUP);
    println!("{:04o}  {}  after chmod g-w", p.bits(), render(p));

    // chmod u+s (setuid).
    p.enable(SETUID);
    println!("{:04o}  {}  after chmod u+s (setuid={})",
        p.bits(), render(p), p.has(SETUID));

    // Quick queries.
    println!("\nworld-writable? {}", p.has(W_OTHER));
    println!("owner has full r/w/x? {}", p.has_all(R_OWNER | W_OWNER | X_OWNER));
    println!("any setuid/setgid bit?  {}", p.has_any(SETUID | SETGID));
}
