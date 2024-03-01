#![no_std]
extern crate alloc;
// use alloc::{string::ToString, vec::Vec};
// use range_set_blaze::RangeSetBlaze;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct LedState {
    pub state: u8,
    pub duration: u32,
}

#[wasm_bindgen]
pub fn get_led_state_and_duration(milliseconds: f64) -> LedState {
    let milliseconds = milliseconds as u32;
    let seconds = milliseconds / 1000;
    let state = if seconds % 2 == 0 {
        0b11111111
    } else {
        0b00000000
    };
    let duration = 100;

    LedState { state, duration }
}
