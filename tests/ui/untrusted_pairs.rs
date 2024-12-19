use range_set_blaze::RangeSetBlaze;

fn main() {
    let guaranteed = RangeSetBlaze::from_iter([1..=2, 3..=4, 5..=6]).into_ranges();
    let _range_set_blaze = RangeSetBlaze::from_sorted_disjoint(guaranteed); // yep
    let not_guaranteed = [5..=6, 1..=3, 3..=4].into_iter();
    let _range_set_blaze = RangeSetBlaze::from_sorted_disjoint(not_guaranteed); // nope
}
