use range_set_blaze::RangeSetBlaze;
use wasm_bindgen_test::*;
wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn insert_255u8() {
    let range_set_blaze = RangeSetBlaze::<u8>::from_iter([255]);
    assert!(range_set_blaze.to_string() == "255..=255");
}
