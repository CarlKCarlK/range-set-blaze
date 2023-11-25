// Tell 'nightly' Rust to enable 'portable_simd'
#![feature(portable_simd)]
use core::simd::prelude::*;

// SIMD vector constants
const LANES: usize = 32;
const THIRTEENS: Simd<u8, LANES> = Simd::<u8, LANES>::from_array([13; LANES]);
const TWENTYSIXS: Simd<u8, LANES> = Simd::<u8, LANES>::from_array([26; LANES]);
const ZEES: Simd<u8, LANES> = Simd::<u8, LANES>::from_array([b'Z'; LANES]);

fn main() {
    // create a SIMD vector from a slice of LANES bytes
    let mut data = Simd::<u8, LANES>::from_slice(b"URYYBJBEYQVQBUBCRVGFNYYTBVATJRYY");

    data += THIRTEENS; // add 13 to each byte

    // compare each byte to 'Z', where the byte is greater than 'Z', subtract 26
    let mask = data.simd_gt(ZEES); // compare each byte to 'Z'
    data = mask.select(data - TWENTYSIXS, data);

    let output = String::from_utf8_lossy(data.as_array());
    assert_eq!(output, "HELLOWORLDIDOHOPEITSALLGOINGWELL");
    println!("{}", output);
}
