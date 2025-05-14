//! Code for the article "Nine Rules for Generalizing a Rust Data Structure: Lessons from Extending `RangeSetBlaze` to Support Maps"

fn main() {
    println!("Run: cargo test --example nine_rules_maps -- --nocapture");
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use range_set_blaze::{RangeMapBlaze, RangeSetBlaze};
    #[test]
    fn example_test_1() {
        let set = RangeSetBlaze::from_iter([4, 0, 3, 5, 4]);
        println!("Set: {set:?}");

        let map =
            RangeMapBlaze::from_iter([(4, "green"), (0, "white"), (3, "orange"), (5, "green")]);
        println!("Map: {map:?}");
    }

    #[test]
    fn example_test_2() {
        let a = RangeMapBlaze::from_iter([(0..=3, "green")]);
        let b = RangeMapBlaze::from_iter([(2..=5, "white")]);
        assert_eq!(
            a | b,
            RangeMapBlaze::from_iter([(0..=3, "green"), (2..=5, "white")])
        );
    }

    #[test]
    fn example_test_3() {
        let btree_map = BTreeMap::from_iter([(4, "green"), (0, "white"), (4, "orange")]);
        assert_eq!(btree_map[&4], "orange");
        let range_map = RangeMapBlaze::from_iter([(4, "green"), (0, "white"), (4, "orange")]);
        assert_eq!(range_map[4], "green");
    }
}
