use range_set_blaze::prelude::*;
use std::ops::BitXor;

fn main() {
    let a = RangeSetBlaze::from_iter([1, 2, 3]).into_ranges();
    let b = [(1, 2), (3, 4)].into_iter();
    let _c = a.bitxor(b);
}
