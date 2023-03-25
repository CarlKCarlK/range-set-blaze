use range_set_blaze::RangeSetBlaze;
fn _some_fn() {
    let trusted = RangeSetBlaze::from_iter([1u8..=2, 3..=4, 5..=6]).into_ranges();
    let _range_set_int: RangeSetBlaze<_> = trusted.into();
    let untrusted = [(1, 2), (3, 4), (5, 6)];
    let range_set_blaze: RangeSetBlaze<_> = untrusted.iter().into();
}

fn main() {}
