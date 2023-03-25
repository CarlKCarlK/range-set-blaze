use itertools::Itertools;
use range_set_blaze::{RangeSetBlaze, SortedDisjointIterator};

fn main() {
    let a = [(1, -1), (-3, 4)];
    let (a0, a1) = a.iter().tee();
    let _c = a0.union(a1);
}
