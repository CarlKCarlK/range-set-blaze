//! Code for the article "Nine Rules for Generalizing a Rust Data Structure: Lessons from Extending `RangeSetBlaze` to Support Maps"

fn main() {
    println!("Run: cargo test --example nine_rules_maps -- --nocapture");
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::net::{Ipv4Addr, Ipv6Addr};

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
            RangeMapBlaze::from_iter([(0..=3, "green"), (4..=5, "white")])
        );
    }

    #[test]
    fn example_test_3() {
        let btree_map = BTreeMap::from_iter([(4, "green"), (0, "white"), (4, "orange")]);
        assert_eq!(btree_map[&4], "orange");
        let range_map = RangeMapBlaze::from_iter([(4, "green"), (0, "white"), (4, "orange")]);
        assert_eq!(range_map[4], "green");
    }

    #[test]
    fn example_test_4() {
        let mut map = RangeMapBlaze::from_iter([(b' '..=b'~', "printable")]);
        map |= map.complement_with(&"non-printable"); // in-place union
        println!("map: {:?}", map);
        // Prints: map: (0..=31, "non-printable"), (32..=126, "printable"), (127..=255, "non-printable")
        println!("'tab' is: {:?}", map[b'\t']);
        // Prints: 'tab' is: "non-printable"
    }

    #[test]
    fn example_test_5() {
        // Mind the gap between 55295 and 57344
        let char_range = RangeSetBlaze::from_iter([
            char::from_u32(55295).unwrap()..=char::from_u32(57344).unwrap()
        ]);

        println!(
            "# of characters in inclusive range: {char_range:?} is {:?}",
            char_range.len()
        );
        // Prints: # of characters in inclusive range: '\u{d7ff}'..='\u{e000}' is 2

        // // --- IPv4: subtract one range from a next-hop map ---
        let next_hop_map = RangeMapBlaze::from_iter([(
            Ipv4Addr::new(192, 168, 1, 0)..=Ipv4Addr::new(192, 168, 1, 255),
            Ipv4Addr::new(152, 10, 0, 0),
        )]);
        let set = RangeSetBlaze::from_iter([
            Ipv4Addr::new(192, 168, 1, 100)..=Ipv4Addr::new(192, 168, 1, 200)
        ]);
        println!("IPv4 diff: {:?}", &next_hop_map - &set);
        // Prints: IPv4 diff: (192.168.1.0..=192.168.1.99, 152.10.0.0), (192.168.1.201..=192.168.1.255, 152.10.0.0)

        // --- IPv6: count all addresses via complement of the empty set ---
        let full = !RangeSetBlaze::<Ipv6Addr>::default();
        println!("IPv6 address count: {:?}", full.len());
        // Prints: IPv6 address count: MaxPlusOne (which is u128::MAX + 1)
    }

    #[derive(Debug, Eq, PartialEq)]
    struct Big(&'static str);
    impl Clone for Big {
        fn clone(&self) -> Self {
            let Big(name) = self;
            println!("Cloned: {:?}", name);
            Big(name)
        }
    }

    #[test]
    fn example_test_6() {
        let a = RangeMapBlaze::from_iter([(0..=10, Big("green"))]);
        let b = RangeMapBlaze::from_iter([(11..=15, Big("green")), (99..=99, Big("yellow"))]);
        // Inputs not owned, so must clone Big's
        let c = &a | &b;
        println!("{c:?}");
        // Prints: (0..=15, Big("green")), (99..=99, Big("yellow"))
        // Inputs owned, so values are moved, not cloned.
        let d = a | b;
        println!("{d:?}");
        // Prints: (0..=15, Big("green")), (99..=99, Big("yellow"))
        assert_eq!(
            format!("{c:?}"),
            r#"(0..=15, Big("green")), (99..=99, Big("yellow"))"#
        );
        assert_eq!(
            format!("{d:?}"),
            r#"(0..=15, Big("green")), (99..=99, Big("yellow"))"#
        );
    }

    #[test]
    fn example_test_7() {
        // Imagine Big is expensive to clone, like a struct.
        // Requires only one clone.
        let a = RangeMapBlaze::from_iter([(0..=10, Big("green")), (5..=5, Big("white"))]);
        assert_eq!(
            format!("{a:?}"),
            r#"(0..=4, Big("green")), (5..=5, Big("white")), (6..=10, Big("green"))"#
        );
    }

    #[test]
    fn example_test_8() {
        let printable = RangeSetBlaze::from_iter([b' '..=b'~']);
        let unprintable = !printable;
        assert_eq!(unprintable.to_string(), "0..=31, 127..=255");

        let printable_map = RangeMapBlaze::from_iter([(b' '..=b'~', "printable")]);
        let unprintable_set = !printable_map;
        assert_eq!(unprintable_set.to_string(), "0..=31, 127..=255");
    }

    #[test]
    fn example_test_9() {
        let a = RangeMapBlaze::from_iter([(0..=10, "green")]);
        let map = RangeMapBlaze::from_iter([(3..=3, "green"), (9..=100, "yellow")]);
        assert_eq!(
            &a - &map,
            RangeMapBlaze::from_iter([(0..=2, "green"), (4..=8, "green")])
        );
        let set = RangeSetBlaze::from_iter([3..=3, 9..=100]);
        assert_eq!(
            &a - &set,
            RangeMapBlaze::from_iter([(0..=2, "green"), (4..=8, "green")])
        );
    }

    // Map A: 0..=3 → "green"
    // Map B: 2..=5 → "white"
    // Map C: A ∩ B : 2..=3 → ????

    #[test]
    fn example_test_10() {
        let a = RangeMapBlaze::from_iter([(0..=3, "green")]);
        let b = RangeMapBlaze::from_iter([(2..=5, "white")]);
        println!("{:?}", a & b); // intersection
    }

    #[test]
    fn example_test_11() {
        use range_set_blaze::IntoString;
        use range_set_blaze::SortedDisjoint;

        let a = RangeSetBlaze::from_iter([0u8..=2]);
        let b = RangeSetBlaze::from_iter([1u8..=3]);
        println!("a        : {}", a);
        println!("b        : {}", b);
        println!("!a       : {}", (!(a.ranges())).into_string());
        println!("!b       : {}", (!(b.ranges())).into_string());
        println!(
            "!a|!b    : {}",
            (!(a.ranges()) | !(b.ranges())).into_string()
        );
        println!(
            "!(!a|!b) : {}",
            (!(!(a.ranges()) | !(b.ranges()))).into_range_set_blaze()
        );
    }

    #[test]
    fn example_test_12() {
        let a = RangeMapBlaze::from_iter([(0u8..=1, "white"), (2..=2, "green")]);
        let b = RangeMapBlaze::from_iter([(1u8..=1, "blue"), (2..=3, "yellow")]);
        println!("a   : {}", a);
        println!("b   : {}", b);
        println!("a&b : {}", a & b);
    }

    #[test]
    fn understand_bench_1() {
        use rand::SeedableRng;
        use rand::rngs::StdRng;
        use range_set_blaze::test_util::{How, k_maps};

        let range = 0..=99_999_999u32;
        let range_len_list = [1, 10, 100, 1000, 10_000, 100_000, 1_000_000];
        let coverage_goal_list = [0.1];
        let how = How::None;
        let seed = 0;
        let value_count = 5u32;
        let range_per_clump = 1; // making this 1 or 100 changes nothing.

        let mut rng = StdRng::seed_from_u64(seed);

        for coverage_goal in coverage_goal_list {
            for range_len in &range_len_list {
                let map0 = k_maps(
                    2,
                    *range_len,
                    &range,
                    coverage_goal,
                    how,
                    &mut rng,
                    value_count,
                    range_per_clump,
                )
                .pop()
                .expect("real assert");

                let fraction = map0.len() as f64 / (range.end() + 1) as f64;
                println!(
                    "coverage_goal {coverage_goal}, range_len {range_len}, fraction {fraction}"
                );
            }
        }
    }
}
