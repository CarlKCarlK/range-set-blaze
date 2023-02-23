use range_set_int::{RangeSetInt, SortedDisjointIterator};
// !!!cmk should users use a prelude?
fn _some_fn() {
    let trusted = RangeSetInt::<u8>::from("1..=2,3..=4,5..=6");
    let trusted = trusted.ranges();
    let _range_set_int: RangeSetInt<_> = trusted.into();
    let untrusted = [(1, 2), (3, 4), (5, 6)];
    let range_set_int: RangeSetInt<_> = untrusted.iter().into();
}

fn main() {}
