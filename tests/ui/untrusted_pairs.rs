use range_set_blaze::RangeSetBlaze;

fn _some_fn() {
    let trusted = RangeSetBlaze::from_iter([1..=2, 3..=4, 5..=6]).into_ranges();
    let _range_set_int = RangeSetBlaze::from_sorted_disjoint(trusted);
    let untrusted = [5..=6, 1..=3, 3..=4].into_iter();
    let _range_set_int = RangeSetBlaze::from_sorted_disjoint(untrusted);
}

fn main() {}
