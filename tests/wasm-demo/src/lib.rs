#![no_std]
#![allow(clippy::cargo_common_metadata)]

extern crate alloc;
use alloc::{string::ToString, vec::Vec};
use range_set_blaze::RangeSetBlaze;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn disjoint_intervals(input: Vec<i32>) -> JsValue {
    let set: RangeSetBlaze<_> = input.into_iter().collect();
    let s = set.to_string();
    JsValue::from_str(&s)
}
