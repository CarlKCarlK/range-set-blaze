use range_set_int::{RangeSetInt, SortedDisjointIterator};

fn main() {
    let a = RangeSetInt::from_iter([1, 2, 3]).into_ranges();
    let b = [(1, 2), (3, 4)].into_iter();
    let _c = a.bitxor(b);
}
