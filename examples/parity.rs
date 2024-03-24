use range_set_blaze::prelude::*;

fn main() {
    let a = RangeSetBlaze::from_iter([1..=6, 8..=9, 11..=15]);
    let b = RangeSetBlaze::from_iter([5..=13, 18..=29]);
    let c = RangeSetBlaze::from_iter([38..=42]);

    let parity = union_dyn!(
        intersection_dyn!(a.ranges(), !b.ranges(), !c.ranges()),
        intersection_dyn!(!a.ranges(), b.ranges(), !c.ranges()),
        intersection_dyn!(!a.ranges(), !b.ranges(), c.ranges()),
        intersection_dyn!(a.ranges(), b.ranges(), c.ranges())
    );
    assert_eq!(
        parity.to_string(),
        "1..=4, 7..=7, 10..=10, 14..=15, 18..=29, 38..=42"
    );
}
