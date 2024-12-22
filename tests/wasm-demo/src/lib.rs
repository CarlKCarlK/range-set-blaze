#![no_std]

extern crate alloc;
use alloc::{string::ToString, vec::Vec};
use range_set_blaze::RangeSetBlaze;
use wasm_bindgen::prelude::*;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
pub fn disjoint_intervals(input: Vec<i32>) -> JsValue {
    let set: RangeSetBlaze<_> = input.into_iter().collect();
    let s = set.to_string();
    JsValue::from_str(&s)
}
