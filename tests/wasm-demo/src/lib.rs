use std::io::{BufReader, Cursor};

use range_set_blaze::{demo_read_ranges_from_buffer, RangeSetBlaze};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn disjoint_intervals(input: Vec<i32>) -> JsValue {
    let set: RangeSetBlaze<_> = input.into_iter().collect();
    let s = set.to_string();
    JsValue::from_str(&s)
}

#[wasm_bindgen]
pub fn demo_read_ranges_from_slice(data: &[u8]) -> JsValue {
    let reader = BufReader::new(Cursor::new(data));
    match demo_read_ranges_from_buffer::<_, i32>(reader) {
        Ok(set) => JsValue::from_str(&set.to_string()),
        Err(e) => JsValue::from_str(&format!("Error: {}", e)),
    }
}
