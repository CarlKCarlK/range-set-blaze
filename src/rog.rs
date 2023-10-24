// cmk
// #![cfg(feature = "rog-experimental")]
// #![deprecated(
//     note = "The rog ('range or gap') module is experimental and may be changed or removed in future versions."
// )]

use core::ops::{Bound, RangeBounds, RangeInclusive};

use alloc::collections::btree_map;

use crate::{Integer, RangeSetBlaze};

struct RogsIter<'a, T: Integer> {
    end_in: T,
    first_rog: Option<Rog<T>>,
    gap_start: Option<T>,
    btree_map_range: Option<btree_map::Range<'a, T, T>>,
}

impl<T: Integer> Iterator for RogsIter<'_, T> {
    type Item = Rog<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(rog) = self.first_rog.take() {
            return Some(rog);
        };

        if let Some(btree_map_range) = &mut self.btree_map_range {
            if let Some((start_el, end_el)) = btree_map_range.next() {
                if self.end_in < *start_el {
                    self.btree_map_range = None;
                } else {
                    let result = Rog::Gap(self.gap_start.unwrap()..=*start_el - T::one());
                    if end_el < &self.end_in {
                        self.first_rog = Some(Rog::Range(*start_el..=*end_el));
                        self.gap_start = Some(*end_el + T::one());
                    } else {
                        self.first_rog = Some(Rog::Range(*start_el..=self.end_in));
                        self.gap_start = None;
                    }
                    return Some(result);
                }
            } else {
                self.btree_map_range = None;
            }
        };

        if let Some(gap_start) = self.gap_start.take() {
            return Some(Rog::Gap(gap_start..=self.end_in));
        };

        return None;
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
}

impl<T: Integer> RangeSetBlaze<T> {
    fn rogs_range_slow<R>(&self, range: R) -> Vec<Rog<T>>
    where
        R: RangeBounds<T>,
    {
        // cmk similar code elsewhere
        let (start_in, end_in) = extract_range(range);
        let rsb_in = RangeSetBlaze::from_iter([start_in..=end_in]);
        let ranges = &rsb_in & self;
        let gaps = rsb_in - self;
        let ranges = ranges.ranges().into_iter().map(|r| Rog::Range(r));
        let gaps = gaps.ranges().into_iter().map(|r| Rog::Gap(r));
        let mut result = ranges.chain(gaps).collect::<Vec<Rog<T>>>();
        result.sort_by_key(|a| a.start());
        result
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
    pub fn rogs_range<R>(&self, range: R) -> Vec<Rog<T>>
    where
        R: RangeBounds<T>,
    {
        // cmk similar code elsewhere
        let (start_in, end_in) = extract_range(range);

        let mut before = self.btree_map.range(..=start_in).rev();
        let rogs_iter = if let Some((_, end_before)) = before.next() {
            if end_before < &start_in {
                // case 1: range doesn't touch the before range
                RogsIter {
                    end_in,
                    first_rog: None,
                    gap_start: Some(start_in),
                    btree_map_range: Some(self.btree_map.range(start_in..)),
                }
            } else if end_before < &end_in {
                // case 2: the range touches and extends beyond the before range
                RogsIter {
                    end_in,
                    first_rog: Some(Rog::Range(start_in..=*end_before)),
                    gap_start: Some(*end_before + T::one()), // cmk check all arithmetic
                    btree_map_range: Some(self.btree_map.range(start_in..)),
                }
            } else {
                // case 3 the range is completely contained in the before range
                RogsIter {
                    end_in,
                    first_rog: Some(Rog::Range(start_in..=end_in)),
                    gap_start: None,
                    btree_map_range: None,
                }
            }
        } else {
            // case 4: there is no before range
            RogsIter {
                end_in,
                first_rog: None,
                gap_start: Some(start_in),
                btree_map_range: Some(self.btree_map.range(start_in..)),
            }
        };

        rogs_iter.collect::<Vec<Rog<T>>>()
    }
}

#[inline]
fn extract_range<T: Integer, R>(range: R) -> (T, T)
where
    R: RangeBounds<T>,
{
    let start = match range.start_bound() {
        Bound::Included(n) => *n,
        Bound::Excluded(n) => *n + T::one(),
        Bound::Unbounded => T::min_value(),
    };
    let end = match range.end_bound() {
        Bound::Included(n) => *n,
        Bound::Excluded(n) => *n - T::one(),
        Bound::Unbounded => T::safe_max_value(),
    };
    assert!(start <= end);
    // cmk ok to panic, but give better message (see btreeset)
    (start, end)
}

#[cfg(test)]
mod tests {
    use super::*; // Import the parent module's contents.

    #[test]
    fn test_rog_functionality() {
        let a = RangeSetBlaze::from_iter([1..=6, 8..=9, 11..=15]);
        // case 1:
        for end in 7..=16 {
            println!("case 1: {:?}", a.rogs_range_slow(7..=end));
            assert_eq!(a.rogs_range_slow(7..=end), a.rogs_range(7..=end));
        }
        // case 2:
        for end in 7..=16 {
            println!("case 2: {:?}", a.rogs_range_slow(4..=end));
            assert_eq!(a.rogs_range_slow(4..=end), a.rogs_range(4..=end));
        }
        // case 3:
        for start in 11..=15 {
            for end in start..=15 {
                println!("case 3: {:?}", a.rogs_range_slow(start..=end));
                assert_eq!(a.rogs_range_slow(start..=end), a.rogs_range(start..=end));
            }
        }
        // case 4:
        for end in -1..=16 {
            println!("case 4: {:?}", a.rogs_range_slow(-1..=end));
            assert_eq!(a.rogs_range_slow(-1..=end), a.rogs_range(-1..=end));
        }

        // let rri = a.rogs_range(7..);
        // let rri = a.rogs_range(7..=22);
        // let rri = a.rogs_range(7..=12);
        // // case 3
        // let rri = a.rogs_range(2..=6);
        // // let rri = a.rog_range(10..=20);
        // // assert!(rri.next(), Some(Rog::Gap(10..=10)));
        // // assert!(rri.next(), Some(Rog::Range(11..=15)));
        // // assert!(rri.next(), Some(Rog::Range(16..=20)));
        // // assert!(rri.next(), None);
    }
}
