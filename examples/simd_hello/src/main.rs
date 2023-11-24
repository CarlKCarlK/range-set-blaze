// Tell 'nightly' Rust to enable 'portable_simd'
#![feature(portable_simd)]

use core::simd::prelude::*;

// SIMD vector constants
const THIRTEENS: Simd<u8, 16> = Simd::<u8, 16>::from_array([13; 16]);
const TWENTYSIXS: Simd<u8, 16> = Simd::<u8, 16>::from_array([26; 16]);
const ZEES: Simd<u8, 16> = Simd::<u8, 16>::from_array([b'Z'; 16]);

fn main() {
    // create a SIMD vector from a slice of 16 bytes
    let mut data = Simd::<u8, 16>::from_slice(b"URYYBJBEYQNOPQRS");

    data += THIRTEENS; // add 13 to each byte
    let mask = data.simd_gt(ZEES); // compare each byte to 'Z'

    // where the byte is greater than 'Z', subtract 26
    data = mask.select(data - TWENTYSIXS, data);

    let output = String::from_utf8_lossy(data.as_array());
    assert_eq!(output, "HELLOWORLDABCDEF");
    println!("{}", output);
}
