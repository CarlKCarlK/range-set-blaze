//! cmk000 move
#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn bitor_precedence_test() {
    use range_set_blaze::RangeMapBlaze;

    // Test case 1: Testing BitOr<Self> for RangeMapBlaze (owned + owned)
    {
        let self_map = RangeMapBlaze::from_iter([(1..=2, "a"), (5..=100, "a")]);
        let other_map = RangeMapBlaze::from_iter([(2..=6, "b")]);

        // This should give precedence to values in other_map over self_map
        let result = self_map | other_map;

        assert_eq!(
            result,
            RangeMapBlaze::from_iter([(1..=1, "a"), (2..=6, "b"), (7..=100, "a")])
        );
    }

    // Test case 2: Testing BitOr<&Self> for RangeMapBlaze (owned + borrowed)
    {
        let self_map = RangeMapBlaze::from_iter([(1..=2, "a"), (5..=100, "a")]);
        let other_map = RangeMapBlaze::from_iter([(2..=6, "b")]);

        // This should give precedence to values in other_map over self_map
        let result = self_map | &other_map;

        assert_eq!(
            result,
            RangeMapBlaze::from_iter([(1..=1, "a"), (2..=6, "b"), (7..=100, "a")])
        );
    }

    // Test case 3: Testing BitOr<RangeMapBlaze<T, V>> for &RangeMapBlaze<T, V> (borrowed + owned)
    {
        let self_map = RangeMapBlaze::from_iter([(1..=2, "a"), (5..=100, "a")]);
        let other_map = RangeMapBlaze::from_iter([(2..=6, "b")]);

        // This should give precedence to values in other_map over self_map
        let result = &self_map | other_map;

        assert_eq!(
            result,
            RangeMapBlaze::from_iter([(1..=1, "a"), (2..=6, "b"), (7..=100, "a")])
        );
    }

    // Test case 4: Testing BitOr<&RangeMapBlaze<T, V>> for &RangeMapBlaze<T, V> (borrowed + borrowed)
    {
        let self_map = RangeMapBlaze::from_iter([(1..=2, "a"), (5..=100, "a")]);
        let other_map = RangeMapBlaze::from_iter([(2..=6, "b")]);

        // This should give precedence to values in other_map over self_map
        let result = &self_map | &other_map;

        assert_eq!(
            result,
            RangeMapBlaze::from_iter([(1..=1, "a"), (2..=6, "b"), (7..=100, "a")])
        );
    }

    // Additional test for the fast_union test case
    {
        let a = RangeMapBlaze::from_iter([(1..=2, "a")]);
        let b = RangeMapBlaze::from_iter([
            (1..=5, "x"),
            (13..=14, "b"),
            (15..=16, "c"),
            (17..=18, "d"),
            (19..=20, "e"),
        ]);
        let c = a | b;

        // Values from b should have precedence
        assert_eq!(
            c.to_string(),
            r#"(1..=5, "x"), (13..=14, "b"), (15..=16, "c"), (17..=18, "d"), (19..=20, "e")"#
        );
    }
}
