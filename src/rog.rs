// cmk
// #![cfg(feature = "rog-experimental")]
// #![deprecated(
//     note = "The rog ('range or gap') module is experimental and may be changed or removed in future versions."
// )]

use core::ops::{Bound, RangeBounds, RangeInclusive};

use alloc::collections::btree_map;

use crate::{Integer, RangeSetBlaze};

pub struct RogsIter<'a, T: Integer> {
    end_in: T,
    next_rog: Option<Rog<T>>,
    final_gap_start: Option<T>,
    btree_map_iter: btree_map::Range<'a, T, T>,
}

impl<T: Integer> Iterator for RogsIter<'_, T> {
    type Item = Rog<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(rog) = self.next_rog.take() {
            return Some(rog);
        };

        if let Some((start_el, end_el)) = self.btree_map_iter.next() {
            if self.end_in < *start_el {
                self.btree_map_iter = btree_map::Range::default();
            } else {
                debug_assert!(self.final_gap_start.is_some()); // final_gap_start should be Some if we're in this branch
                debug_assert!(self.final_gap_start.unwrap() < *start_el); // so -1 is safe
                let result = Rog::Gap(self.final_gap_start.unwrap()..=*start_el - T::one());
                if end_el < &self.end_in {
                    self.next_rog = Some(Rog::Range(*start_el..=*end_el));
                    debug_assert!(end_el < &self.end_in); // so +1 is safe
                    self.final_gap_start = Some(*end_el + T::one());
                } else {
                    self.next_rog = Some(Rog::Range(*start_el..=self.end_in));
                    self.final_gap_start = None;
                }
                return Some(result);
            }
        };

        if let Some(gap_start) = self.final_gap_start.take() {
            return Some(Rog::Gap(gap_start..=self.end_in));
        };

        None
    }
}

/// Enum to represent either a range or a gap.
#[derive(Debug, PartialEq)]
pub enum Rog<T: Integer> {
    Range(RangeInclusive<T>),
    Gap(RangeInclusive<T>),
}

impl<T: Integer> Rog<T> {
    pub fn start(&self) -> T {
        match self {
            Rog::Range(r) => *r.start(),
            Rog::Gap(r) => *r.start(),
        }
    }
    pub fn end(&self) -> T {
        match self {
            Rog::Range(r) => *r.end(),
            Rog::Gap(r) => *r.end(),
        }
    }
    pub fn contains(&self, value: T) -> bool {
        match self {
            Rog::Range(r) => r.contains(&value),
            Rog::Gap(r) => r.contains(&value),
        }
    }
}

impl<T: Integer> RangeSetBlaze<T> {
    pub fn rogs_at(&self, at: T) -> Rog<T> {
        assert!(
            at <= T::safe_max_value(),
            "at must be <= T::safe_max_value()"
        );
        let mut before = self.btree_map.range(..=at).rev();
        if let Some((start_before, end_before)) = before.next() {
            if end_before < &at {
                // case 1: range doesn't touch the before range
                let start_out = *end_before + T::one();
                if let Some((start_next, _)) = self.btree_map.range(at..).next() {
                    debug_assert!(start_before < start_next); // so -1 is safe
                    Rog::Gap(start_out..=*start_next - T::one())
                } else {
                    Rog::Gap(start_out..=T::safe_max_value())
                }
            } else {
                // case 2&3: the range touches the before range
                Rog::Range(*start_before..=*end_before)
            }
        } else {
            // case 4: there is no before range
            if let Some((start_next, _)) = self.btree_map.range(at..).next() {
                debug_assert!(at < *start_next); // so -1 is safe
                Rog::Gap(T::min_value()..=*start_next - T::one())
            } else {
                Rog::Gap(T::min_value()..=T::safe_max_value())
            }
        }
    }

    /// Constructs an iterator over a sub-range of elements in the set. The iterator will yield
    /// Rogs, which are either a range or a gap.
    ///
    /// cmk update based on docs in https://doc.rust-lang.org/std/collections/struct.BTreeSet.html#method.range
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::RangeSetBlaze;
    ///
    /// let mut set = RangeSetBlaze::new();
    /// set.insert(3);
    /// set.insert(5);
    /// set.insert(8);
    /// for &elem in set.range(4..=8) {
    ///     println!("{elem}");
    /// }
    /// assert_eq!(Some(&5), set.range(4..).next());
    /// ```
    pub fn rogs_range<R>(&self, range: R) -> RogsIter<T>
    where
        R: RangeBounds<T>,
    {
        let (start_in, end_in) = extract_range(range);

        let mut before = self.btree_map.range(..=start_in).rev();
        if let Some((_, end_before)) = before.next() {
            if end_before < &start_in {
                // case 1: range doesn't touch the before range
                RogsIter {
                    end_in,
                    next_rog: None,
                    final_gap_start: Some(start_in),
                    btree_map_iter: self.btree_map.range(start_in..),
                }
            } else if end_before < &end_in {
                // case 2: the range touches and extends beyond the before range
                debug_assert!(*end_before < end_in); // so +1 is safe
                debug_assert!(start_in <= *end_before); // so +1 is safe
                RogsIter {
                    end_in,
                    next_rog: Some(Rog::Range(start_in..=*end_before)),
                    final_gap_start: Some(*end_before + T::one()),
                    btree_map_iter: self.btree_map.range(start_in + T::one()..),
                }
            } else {
                // case 3 the range is completely contained in the before range
                RogsIter {
                    end_in,
                    next_rog: Some(Rog::Range(start_in..=end_in)),
                    final_gap_start: None,
                    btree_map_iter: btree_map::Range::default(),
                }
            }
        } else {
            // case 4: there is no before range
            RogsIter {
                end_in,
                next_rog: None,
                final_gap_start: Some(start_in),
                btree_map_iter: self.btree_map.range(start_in..),
            }
        }
    }
}

fn extract_range<T: Integer, R>(range: R) -> (T, T)
where
    R: RangeBounds<T>,
{
    let start = match range.start_bound() {
        Bound::Included(n) => *n,
        Bound::Excluded(n) => {
            assert!(
                *n < T::safe_max_value(),
                "inclusive start must be <= T::max_safe_value()"
            );
            *n + T::one()
        }
        Bound::Unbounded => T::min_value(),
    };
    let end = match range.end_bound() {
        Bound::Included(n) => *n,
        Bound::Excluded(n) => {
            assert!(
                *n > T::min_value(),
                "inclusive end must be >= T::min_value()"
            );
            *n - T::one()
        }
        Bound::Unbounded => T::safe_max_value(),
    };
    assert!(start <= end, "start must be <= end");
    assert!(
        end <= T::safe_max_value(),
        "end must be <= T::safe_max_value()"
    );

    (start, end)
}

#[cfg(test)]
mod tests {
    use std::panic::{self, AssertUnwindSafe};

    use super::*; // Import the parent module's contents.

    impl<T: Integer> RangeSetBlaze<T> {
        fn rogs_range_slow<R>(&self, range: R) -> Vec<Rog<T>>
        where
            R: RangeBounds<T>,
        {
            let (start_in, end_in) = extract_range(range);
            let rsb_in = RangeSetBlaze::from_iter([start_in..=end_in]);
            let ranges = &rsb_in & self;
            let gaps = rsb_in - self;
            let ranges = ranges.ranges().map(|r| Rog::Range(r));
            let gaps = gaps.ranges().map(|r| Rog::Gap(r));
            let mut result = ranges.chain(gaps).collect::<Vec<Rog<T>>>();
            result.sort_by_key(|a| a.start());
            result
        }

        fn rogs_at_slow(&self, at: T) -> Rog<T> {
            assert!(
                at <= T::safe_max_value(),
                "at must be <= T::safe_max_value()"
            );
            let all_rogs = self.rogs_range_slow(..);
            for rog in all_rogs {
                if rog.contains(at) {
                    return rog;
                }
            }
            unreachable!("at must be in something");
        }
    }

    #[test]
    fn test_rog_functionality() {
        let a = RangeSetBlaze::from_iter([1..=6, 8..=9, 11..=15]);
        // case 1:
        for end in 7..=16 {
            println!("case 1: {:?}", a.rogs_range_slow(7..=end));
            assert_eq!(
                a.rogs_range_slow(7..=end),
                a.rogs_range(7..=end).collect::<Vec<_>>()
            );
        }
        // case 2:
        for end in 7..=16 {
            println!("case 2: {:?}", a.rogs_range_slow(4..=end));
            assert_eq!(
                a.rogs_range_slow(4..=end),
                a.rogs_range(4..=end).collect::<Vec<_>>()
            );
        }
        // case 3:
        for start in 11..=15 {
            for end in start..=15 {
                println!("case 3: {:?}", a.rogs_range_slow(start..=end));
                assert_eq!(
                    a.rogs_range_slow(start..=end),
                    a.rogs_range(start..=end).collect::<Vec<_>>()
                );
            }
        }
        // case 4:
        for end in -1..=16 {
            println!("case 4: {:?}", a.rogs_range_slow(-1..=end));
            assert_eq!(
                a.rogs_range_slow(-1..=end),
                a.rogs_range(-1..=end).collect::<Vec<_>>()
            );
        }
    }

    #[test]
    fn test_rogs_at_functionality() {
        let a = RangeSetBlaze::from_iter([1..=6, 8..=9, 11..=15]);
        for at in 0..=16 {
            println!("{:?}", a.rogs_at_slow(at));
            assert_eq!(a.rogs_at_slow(at), a.rogs_at(at));
        }
    }
    #[test]
    fn test_rog_repro1() {
        let a = RangeSetBlaze::from_iter([1u8..=6u8]);
        assert_eq!(
            a.rogs_range_slow(1..=7),
            a.rogs_range(1..=7).collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_rog_repro2() {
        let a = RangeSetBlaze::from_iter([1..=6, 8..=9, 11..=15]);
        assert_eq!(
            a.rogs_range_slow(4..=8),
            a.rogs_range(4..=8).collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_rog_coverage1() {
        let a = RangeSetBlaze::from_iter([1u8..=6u8]);
        assert!(panic::catch_unwind(AssertUnwindSafe(
            || a.rogs_range((Bound::Excluded(&255), Bound::Included(&255)))
        ))
        .is_err());
        assert!(panic::catch_unwind(AssertUnwindSafe(|| a.rogs_range(0..0))).is_err());
    }

    #[test]
    fn test_rog_extremes_u8() {
        for a in [
            RangeSetBlaze::from_iter([1u8..=6u8]),
            RangeSetBlaze::from_iter([0u8..=6u8]),
            RangeSetBlaze::from_iter([200u8..=255u8]),
            RangeSetBlaze::from_iter([0u8..=255u8]),
            RangeSetBlaze::from_iter([0u8..=5u8, 20u8..=255]),
        ] {
            for start in 0u8..=255 {
                for end in start..=255 {
                    println!("{start}..={end}");
                    assert_eq!(
                        a.rogs_range_slow(start..=end),
                        a.rogs_range(start..=end).collect::<Vec<_>>()
                    );
                }
            }
        }
    }

    #[test]
    fn test_rog_at_extremes_u8() {
        for a in [
            RangeSetBlaze::from_iter([1u8..=6u8]),
            RangeSetBlaze::from_iter([0u8..=6u8]),
            RangeSetBlaze::from_iter([200u8..=255u8]),
            RangeSetBlaze::from_iter([0u8..=255u8]),
            RangeSetBlaze::from_iter([0u8..=5u8, 20u8..=255]),
        ] {
            for at in 0u8..=255 {
                println!("{at}");
                assert_eq!(a.rogs_at_slow(at), a.rogs_at(at));
            }
        }
    }

    #[test]
    fn test_rog_extremes_i128() {
        for a in [
            RangeSetBlaze::from_iter([1i128..=6i128]),
            RangeSetBlaze::from_iter([i128::MIN..=6]),
            RangeSetBlaze::from_iter([200..=i128::MAX - 1]),
            RangeSetBlaze::from_iter([i128::MIN..=i128::MAX - 1]),
            RangeSetBlaze::from_iter([i128::MIN..=5, 20..=i128::MAX - 1]),
        ] {
            for start in [i128::MIN, i128::MIN + 1, 0, i128::MAX - 2, i128::MAX - 1] {
                for end in [i128::MIN, i128::MIN + 1, 0, i128::MAX - 2, i128::MAX - 1] {
                    if end < start {
                        continue;
                    }
                    println!("{start}..={end}");
                    assert_eq!(
                        a.rogs_range_slow(start..=end),
                        a.rogs_range(start..=end).collect::<Vec<_>>()
                    );
                }
            }
        }
    }

    #[test]
    fn test_rog_extremes_at_i128() {
        for a in [
            RangeSetBlaze::from_iter([1i128..=6i128]),
            RangeSetBlaze::from_iter([i128::MIN..=6]),
            RangeSetBlaze::from_iter([200..=i128::MAX - 1]),
            RangeSetBlaze::from_iter([i128::MIN..=i128::MAX - 1]),
            RangeSetBlaze::from_iter([i128::MIN..=5, 20..=i128::MAX - 1]),
        ] {
            for at in [i128::MIN, i128::MIN + 1, 0, i128::MAX - 2, i128::MAX - 1] {
                println!("{at}");
                assert_eq!(a.rogs_at_slow(at), a.rogs_at(at));
            }
        }
    }

    #[test]
    fn test_rog_should_fail_i128() {
        for a in [
            RangeSetBlaze::from_iter([1i128..=6i128]),
            RangeSetBlaze::from_iter([i128::MIN..=6]),
            RangeSetBlaze::from_iter([200..=i128::MAX - 1]),
            RangeSetBlaze::from_iter([i128::MIN..=i128::MAX - 1]),
            RangeSetBlaze::from_iter([i128::MIN..=5, 20..=i128::MAX - 1]),
        ] {
            for start in [i128::MIN, i128::MIN + 1, 0, i128::MAX - 1, i128::MAX] {
                for end in [i128::MIN, i128::MIN + 1, 0, i128::MAX - 1, i128::MAX] {
                    if end < start {
                        continue;
                    }
                    println!("{start}..={end}");
                    let slow =
                        panic::catch_unwind(AssertUnwindSafe(|| a.rogs_range_slow(start..=end)))
                            .ok();
                    let fast = panic::catch_unwind(AssertUnwindSafe(|| {
                        a.rogs_range(start..=end).collect::<Vec<_>>()
                    }))
                    .ok();
                    assert_eq!(slow, fast,);
                }
            }
        }
    }

    #[test]
    fn test_rog_at_should_fail_i128() {
        for a in [
            RangeSetBlaze::from_iter([1i128..=6i128]),
            RangeSetBlaze::from_iter([i128::MIN..=6]),
            RangeSetBlaze::from_iter([200..=i128::MAX - 1]),
            RangeSetBlaze::from_iter([i128::MIN..=i128::MAX - 1]),
            RangeSetBlaze::from_iter([i128::MIN..=5, 20..=i128::MAX - 1]),
        ] {
            for at in [i128::MIN, i128::MIN + 1, 0, i128::MAX - 1, i128::MAX] {
                println!("{at}");
                let slow = panic::catch_unwind(AssertUnwindSafe(|| a.rogs_at_slow(at))).ok();
                let fast = panic::catch_unwind(AssertUnwindSafe(|| a.rogs_at(at))).ok();
                assert_eq!(slow, fast,);
            }
        }
    }
}
