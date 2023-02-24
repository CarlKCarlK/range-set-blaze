use range_set_int::{RangeSetInt, SortedDisjointIterator};

fn main() {
    let a = RangeSetInt::from([1, 2, 3]);
    let a = a.ranges();
    let b = [(1, 2), (3, 4)].into_iter();
    let _c = a.bitxor(b);
}
