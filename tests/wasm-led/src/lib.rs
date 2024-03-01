#![no_std]
extern crate alloc;
use range_set_blaze::prelude::*;
use wasm_bindgen::prelude::*;

#[macro_use]
extern crate lazy_static;

#[wasm_bindgen]
pub struct LedState {
    pub state: u8,
    pub duration: i32,
}

lazy_static! {
    static ref COMPILED_MOVIE: RangeMapBlaze<i32, u8> = compile_movie();
}

#[wasm_bindgen(start)]
pub fn on_module_load() {
    let _ = COMPILED_MOVIE;
}

struct Leds;

#[allow(dead_code)]
impl Leds {
    const SEG_A: u8 = 0b00000001;
    const SEG_B: u8 = 0b00000010;
    const SEG_C: u8 = 0b00000100;
    const SEG_D: u8 = 0b00001000;
    const SEG_E: u8 = 0b00010000;
    const SEG_F: u8 = 0b00100000;
    const SEG_G: u8 = 0b01000000;
    const DECIMAL: u8 = 0b10000000;

    const DIGITS: [u8; 10] = [
        0b00111111, // Digit 0
        0b00000110, // Digit 1
        0b01011011, // Digit 2
        0b01001111, // Digit 3
        0b01100110, // Digit 4
        0b01101101, // Digit 5
        0b01111101, // Digit 6
        0b00000111, // Digit 7
        0b01111111, // Digit 8
        0b01101111, // Digit 9
    ];
    const SPACE: u8 = 0b00000000;
}

const FPS: i32 = 24;

// i32 means we can only go 3 weeks at a time at 24fps. Be sure the code checks this.
fn compile_movie() -> RangeMapBlaze<i32, u8> {
    let length_seconds = 30;
    let frame_count = FPS * length_seconds;

    // The `main`` track starts with 15 seconds of black
    let mut main = RangeMapBlaze::from_iter([(0..=frame_count - 1, Leds::SPACE)]);
    // println!("main {main:?}");

    // Create a 10 frame `digits` track with "0" to "9"".
    let mut digits =
        RangeMapBlaze::from_iter((0i32..=9).map(|i| (i..=i, Leds::DIGITS[i as usize])));

    // Make frame 0 be "start"
    digits.insert(0, Leds::DECIMAL);

    // Oops, we've changed our mind and now don't want frames 8 and 9.
    digits = digits - RangeSetBlaze::from_iter([8..=9]);

    // Apply the following linear transformation to `digits``:
    // 1. Make each original frame last one second
    // 2. Reverse the order of the frames
    // 3. Shift the frames 1 second into the future
    digits = linear(&digits, -FPS, FPS);
    // println!("digits m {digits:?}");

    // Composite these together (listed from top to bottom)
    //  1. `digits``
    //  2. `digits` shifted 10 seconds into the future
    //  3. `main`
    main = &digits | &linear(&digits, 1, 10 * FPS) | &main;
    main
}

// cmk try to make generic?
// cmk linear could be a method on RangeMapBlaze
pub fn linear(
    range_map_blaze: &RangeMapBlaze<i32, u8>,
    scale: i32,
    shift: i32,
) -> RangeMapBlaze<i32, u8> {
    if range_map_blaze.is_empty() {
        return RangeMapBlaze::new();
    }

    let first = range_map_blaze.first_key_value().unwrap().0;
    let last = range_map_blaze.last_key_value().unwrap().0;

    range_map_blaze
        .range_values()
        .map(|range_value| {
            let (start, end) = range_value.range.clone().into_inner();
            let mut a = (start - first) * scale.abs() + first;
            let mut b = (end + 1 - first) * scale.abs() + first - 1;
            let last = (last + 1 - first) * scale.abs() + first - 1;
            if scale < 0 {
                (a, b) = (last - b + first, last - a + first);
            }
            let new_range = a + shift..=b + shift;
            (new_range, range_value.value)
        })
        .collect()
}

#[wasm_bindgen]
pub fn get_led_state_and_duration(milliseconds: f64) -> LedState {
    let index = (milliseconds * FPS as f64 / 1000.0) as i32;
    let state = *COMPILED_MOVIE.get(index).unwrap_or(&Leds::DECIMAL);
    let duration = 1000 / FPS;

    LedState { state, duration }
}
