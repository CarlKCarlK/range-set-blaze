#![no_std]
extern crate alloc;
use alloc::format;
use range_set_blaze::prelude::*;
use wasm_bindgen::prelude::*;
use web_sys::console;

#[macro_use]
extern crate lazy_static;

#[wasm_bindgen]
pub struct LedState {
    pub state: u8,
    pub duration: i32,
}

lazy_static! {
    static ref COMPILED_MOVIES: [RangeMapBlaze<i32, u8>; 3] =
        [hello_world(), circles(), double_count_down()];
}

#[wasm_bindgen(start)]
pub fn on_module_load() {
    let _ = COMPILED_MOVIES;
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

    const ASCII_TABLE: [u8; 128] = [
        // Control characters (0-31) + space (32)
        0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, // 0-4
        0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, // 5-9
        0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, // 10-14
        0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, // 15-19
        0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, //  20-24
        0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, //  25-29
        0b00000000, 0b00000000, 0b00000000, // 30-32
        // Symbols (33-47)
        0b10000110, // !
        0b00000000, // "
        0b00000000, // #
        0b00000000, // $
        0b00000000, // %
        0b00000000, // &
        0b00000000, // '
        0b00000000, // (
        0b00000000, // )
        0b00000000, // *
        0b00000000, // +
        0b00000000, // ,
        0b01000000, // -
        0b10000000, // .
        0b00000000, // /
        // Numbers (48-57)
        0b00111111, // 0
        0b00000110, // 1
        0b01011011, // 2
        0b01001111, // 3
        0b01100110, // 4
        0b01101101, // 5
        0b01111101, // 6
        0b00000111, // 7
        0b01111111, // 8
        0b01101111, // 9
        // Symbols (58-64)
        0b00000000, // :
        0b00000000, // ;
        0b00000000, // <
        0b00000000, // =
        0b00000000, // >
        0b00000000, // ?
        0b00000000, // @
        // Uppercase letters (65-90)
        0b01110111, // A
        0b01111100, // B (same as b)
        0b00111001, // C
        0b01011110, // D (same as d)
        0b01111001, // E
        0b01110001, // F
        0b00111101, // G (same as 9)
        0b01110100, // H
        0b00000110, // I (same as 1)
        0b00011110, // J
        0b01110110, // K (approximation)
        0b00111000, // L
        0b00010101, // M (arbitrary, no good match)
        0b01010100, // N
        0b00111111, // O (same as 0)
        0b01110011, // P
        0b01100111, // Q
        0b01010000, // R
        0b01101101, // S (same as 5)
        0b01111000, // T
        0b00111110, // U
        0b00101010, // V (arbitrary, no good match)
        0b00011101, // W (arbitrary, no good match)
        0b01110110, // X (same as H)
        0b01101110, // Y
        0b01011011, // Z (same as 2)
        // Symbols (91-96)
        0b00111001, // [
        0b00000000, // \
        0b00001111, // ]
        0b00000000, // ^
        0b00001000, // _
        0b00000000, // `
        // Lowercase letters (97-122), reusing uppercase for simplicity
        0b01110111, // A
        0b01111100, // B (same as b)
        0b00111001, // C
        0b01011110, // D (same as d)
        0b01111001, // E
        0b01110001, // F
        0b00111101, // G (same as 9)
        0b01110100, // H
        0b00000110, // I (same as 1)
        0b00011110, // J
        0b01110110, // K (approximation)
        0b00111000, // L
        0b00010101, // M (arbitrary, no good match)
        0b01010100, // N
        0b00111111, // O (same as 0)
        0b01110011, // P
        0b01100111, // Q
        0b01010000, // R
        0b01101101, // S (same as 5)
        0b01111000, // T
        0b00111110, // U
        0b00101010, // V (arbitrary, no good match)
        0b00011101, // W (arbitrary, no good match)
        0b01110110, // X (same as H)
        0b01101110, // Y
        0b01011011, // Z (same as 2)
        // Placeholder for simplicity
        0b00111001, // '{' (123)
        0b00000110, // '|' (124)
        0b00001111, // '}' (125)
        0b01000000, // '~' (126)
        0b00000000, // delete (127)
    ];
}

const FPS: i32 = 24;

pub fn hello_world() -> RangeMapBlaze<i32, u8> {
    let message = "321 Hello world!";
    let message: RangeMapBlaze<i32, u8> = message
        .chars()
        .enumerate()
        .map(|(i, c)| (i as i32, Leds::ASCII_TABLE[c as usize]))
        .collect();
    let message = linear(&message, FPS, 0);
    // add gaps of 3 frames between each character
    let message = message
        .range_values()
        .enumerate()
        .map(|(i, range_value)| {
            let (start, end) = range_value.range.clone().into_inner();
            let new_range = start + i as i32 * 3..=end + i as i32 * 3;
            (new_range, range_value.value)
        })
        .collect();
    message
}
pub fn circles() -> RangeMapBlaze<i32, u8> {
    // Light up segments A to F
    let circle = RangeMapBlaze::from_iter([
        (0, Leds::SEG_A),
        (1, Leds::SEG_B),
        (2, Leds::SEG_C),
        (3, Leds::SEG_D),
        (4, Leds::SEG_E),
        (5, Leds::SEG_F),
    ]);
    let mut main = RangeMapBlaze::new();
    let mut scale = 1;
    while scale < 24 {
        // Slow down the circle by a factor of 1 to 24, appending to `main` each time.
        main = &main | linear(&circle, scale, main.len() as i32);
        scale *= 2;
    }
    // append main with itself, but reversed
    main = &main | linear(&main, -1, main.len() as i32);

    // append 10 copies of the fast circle
    for _ in 0..20 {
        main = &main | linear(&circle, -1, main.len() as i32);
    }

    main
}

// i32 means we can only go 3 weeks at a time at 24fps. Be sure the code checks this.
pub fn double_count_down() -> RangeMapBlaze<i32, u8> {
    let length_seconds = 30;
    let frame_count = FPS * length_seconds;

    // The `main`` track starts with 15 seconds of black
    let mut main = RangeMapBlaze::from_iter([(0..=frame_count - 1, Leds::SPACE)]);
    // println!("main {main:?}");

    // Create a 10 frame `digits` track with "0" to "9"".
    let mut digits =
        RangeMapBlaze::from_iter((0i32..=9).map(|i| (i..=i, Leds::DIGITS[i as usize])));

    // Make frame 0 be the middle LED segment.
    digits.insert(0, Leds::SEG_G);

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
pub fn get_led_state_and_duration(movie_id: f64, now_milliseconds: f64) -> LedState {
    let movie = &COMPILED_MOVIES[movie_id as usize]; // cmk check bounds

    // Find what frame for 'now'
    let frame_index = (now_milliseconds * FPS as f64 / 1000.0) as i32;
    console::log_1(&format!("rust: now_milliseconds: {}", now_milliseconds).into());
    console::log_1(&format!("rust: frame_index: {}", frame_index).into());

    // Create a time interval from now to 2 weeks from now
    let now_to_2_weeks = RangeSetBlaze::from_iter([frame_index..=i32::MAX]);

    // Create trim the movie to the time interval
    let now_to_end_of_movie = movie & &now_to_2_weeks;

    // Find the first region in the time interval (if any)
    let first_region_if_any = now_to_end_of_movie.range_values().next();

    // If there is no region (the movie is over), display "." display "." for 2 weeks.
    let Some(range_value) = first_region_if_any else {
        return LedState {
            state: Leds::DECIMAL,
            duration: i32::MAX,
        };
    };

    // Is "now" in this region? If not then we are in a gap. Display "." until a region is ready to display
    if frame_index < *range_value.range.start() {
        return LedState {
            state: Leds::DECIMAL,
            duration: (range_value.range.start() - frame_index) * 1000 / FPS,
        };
    }

    // If we are in a region, compute its duration in milliseconds and the frame to display
    let duration = (range_value.range.end() + 1 - frame_index) * 1000 / FPS;
    LedState {
        state: *range_value.value,
        duration,
    }
}
