#![feature(alloc_error_handler)]
#![no_main]
#![no_std]
// See https://docs.rust-embedded.org/book/collections/index.html?highlight=alloc_error_handler#using-alloc

extern crate alloc;
use alloc::{string::ToString, vec::Vec};
use panic_halt as _;

use core::{alloc::Layout, iter::FromIterator};
use range_set_blaze::RangeSetBlaze;

use alloc_cortex_m::CortexMHeap;
use cortex_m::asm;
use cortex_m_rt::entry;
use cortex_m_semihosting::{debug, hprintln};

// this is the allocator the application will use
#[global_allocator]
static ALLOCATOR: CortexMHeap = CortexMHeap::empty();

const HEAP_SIZE: usize = 1024; // in bytes

#[entry]
fn main() -> ! {
    unsafe { ALLOCATOR.init(cortex_m_rt::heap_start() as usize, HEAP_SIZE) }

    let range_set_blaze = RangeSetBlaze::from_iter([255u8]);
    assert!(range_set_blaze.to_string() == "255..=255");

    let set = RangeSetBlaze::from_iter([1, 2, 2, 3, 4, 4, 5]);
    let mut result = Vec::new();
    for range in set.ranges() {
        result.push(*range.end());
        result.push(*range.start());
    }

    hprintln!("{:?}", result);

    // panic!("will cause endless loop-- only here to test the testing");

    // exit QEMU
    // NOTE do not run this on hardware; it can corrupt OpenOCD state
    debug::exit(debug::EXIT_SUCCESS);

    loop {}
}

// define what happens in an Out Of Memory (OOM) condition
#[alloc_error_handler]
fn alloc_error(_layout: Layout) -> ! {
    asm::bkpt();

    loop {}
}
