// based on https://github.com/rust-embedded/cortex-m-quickstart/blob/master/examples/allocator.rs
// and https://github.com/rust-lang/rust/issues/51540
#![feature(alloc_error_handler)]
#![no_main]
#![no_std]

extern crate alloc;
use alloc::string::ToString;
use alloc_cortex_m::CortexMHeap;
use core::{alloc::Layout, iter::FromIterator};
use cortex_m::asm;
use cortex_m_rt::entry;
use cortex_m_semihosting::{debug, hprintln};
use panic_halt as _;
use range_set_blaze::RangeSetBlaze;

#[global_allocator]
static ALLOCATOR: CortexMHeap = CortexMHeap::empty();
const HEAP_SIZE: usize = 1024; // in bytes

#[entry]
fn main() -> ! {
    unsafe { ALLOCATOR.init(cortex_m_rt::heap_start() as usize, HEAP_SIZE) }

    // test goes here
    let range_set_blaze = RangeSetBlaze::from_iter([100, 103, 101, 102, -3, -4]);
    hprintln!("{:?}", range_set_blaze.to_string());

    // exit QEMU/ NOTE do not run this on hardware; it can corrupt OpenOCD state
    if range_set_blaze.to_string() != "-4..=-3, 100..=103" {
        debug::exit(debug::EXIT_FAILURE);
    }

    debug::exit(debug::EXIT_SUCCESS);
    loop {}
}

#[alloc_error_handler]
fn alloc_error(_layout: Layout) -> ! {
    asm::bkpt();
    loop {}
}
