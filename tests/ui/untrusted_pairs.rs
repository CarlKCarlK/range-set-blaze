use range_set_int::{RangeSetInt, SortedDisjointIterator};
// !!!cmk should users use a prelude?
fn _some_fn() {
    let trusted = RangeSetInt::<u8>::from("1..=2,3..=4,5..=6");
    let trusted = trusted.ranges();
    let _range_set_int = trusted.to_range_set_int();
    let untrusted = [(1, 2), (3, 4), (5, 6)];
    let range_set_int = untrusted.iter().to_range_set_int();
}

fn main() {}
