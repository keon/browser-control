//! Parse the bit fields of an IPv4 header.
//!
//! Run:  `cargo run --example ipv4_header`

use bitkit::{bytes, prelude::*};

fn main() -> Result<(), BitError> {
    // A real IPv4 header (20 bytes): GET request to 8.8.8.8, identification
    // 0x1c46, TTL 64, protocol TCP (6).
    let header: [u8; 20] = [
        0x45, 0x00, 0x00, 0x3c,
        0x1c, 0x46, 0x40, 0x00,
        0x40, 0x06, 0xb1, 0xe6,
        0xc0, 0xa8, 0x00, 0x68,
        0x08, 0x08, 0x08, 0x08,
    ];

    // Byte 0 packs Version (high nibble) and IHL (low nibble).
    let b0 = Bits::<u8>::new(header[0]);
    let version = b0.extract(4..8)?.get();
    let ihl     = b0.extract(0..4)?.get();

    let total_length = bytes::read_u16_be(&header[2..])?;
    let ident        = bytes::read_u16_be(&header[4..])?;

    // Byte 6-7 packs Flags (high 3 bits) and Fragment Offset (low 13 bits).
    let flags_off = Bits::<u16>::new(bytes::read_u16_be(&header[6..])?);
    let flags     = flags_off.extract(13..16)?.get();
    let frag_off  = flags_off.extract(0..13)?.get();

    let ttl       = header[8];
    let protocol  = header[9];
    let checksum  = bytes::read_u16_be(&header[10..])?;
    let src       = bytes::read_u32_be(&header[12..])?;
    let dst       = bytes::read_u32_be(&header[16..])?;

    println!("Version:        {}", version);
    println!("IHL:            {} ({} bytes)", ihl, ihl as usize * 4);
    println!("Total length:   {}", total_length);
    println!("Identification: 0x{:04x}", ident);
    println!("Flags:          {:03b}", flags);
    println!("Frag offset:    {}", frag_off);
    println!("TTL:            {}", ttl);
    println!("Protocol:       {}", protocol);
    println!("Checksum:       0x{:04x}", checksum);
    println!("Source:         {}", ipv4_str(src));
    println!("Destination:    {}", ipv4_str(dst));
    Ok(())
}

fn ipv4_str(x: u32) -> String {
    let b = x.to_be_bytes();
    format!("{}.{}.{}.{}", b[0], b[1], b[2], b[3])
}
